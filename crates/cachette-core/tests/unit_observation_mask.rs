//! Each live unit carries the mask of factions that see it.
//!
//! A faction sees a unit when it sees the tile the unit stands on. The mask
//! projects that relationship into a dense column indexed by unit slot, so
//! queries take one bit test instead of a tile layer lookup.[^1]
//!
//! # References
//!
//! [^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
//! [^2]: Testing rules, drive the real caller. `.agents/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::holding::FactionMask;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// A world wide enough to hold two factions that start out of sight.
const TWO_SIDES: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    ..WorldConfig::DEFAULT
};

/// The primary faction in the fixtures.
const WATCHER: FactionId = FactionId(0);

/// The rival faction in the fixtures.
const STRANGER: FactionId = FactionId(1);

/// A third faction that places no units.
const THIRD_PARTY: FactionId = FactionId(2);

/// The exponent that keeps units stationary.
const KEEP_STILL: u32 = 12;

/// Builds a world where units do not move on their own.
fn a_still_world(config: WorldConfig) -> World {
    let mut world = World::new(config).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world
}

/// Returns an address near the requested one that admits a unit.
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

/// Spawns a soldier at the specified address.
fn a_unit_at(world: &mut World, address: Axial, faction: FactionId) -> Entity {
    world
        .spawn_soldier(address, faction)
        .expect("the fixture places a unit on ground that admits one")
}

/// Returns how many units stand on a tile, whatever any faction can see.
fn truth_units_on(world: &World, address: Axial) -> usize {
    world
        .bridge()
        .count_on_tile(world.soldiers(), address)
        .expect("the derived structure describes the units")
}

/// Finds an empty tile within sight of the given position.
fn watched_empty_ground(world: &World, origin: Axial) -> Axial {
    for ring in 1..=3i32 {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(origin.q + column, origin.r + row);
                if world.admits_a_unit(candidate) && truth_units_on(world, candidate) == 0 {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no empty tile near {origin:?}");
}

#[test]
fn unit_mask_records_observing_factions() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let away = ground_near(&world, Axial::new(80, 80), 8);

    let own = a_unit_at(&mut world, home, WATCHER);
    let rival = a_unit_at(&mut world, away, STRANGER);
    world.step(1).expect("the step runs");

    // The watcher's unit observes its own tile.
    let own_mask = world.unit_mask(own);
    assert!(
        own_mask.contains(WATCHER),
        "a unit is observed by its own faction"
    );
    assert!(
        !own_mask.contains(STRANGER),
        "a distant unit is not observed by the rival faction"
    );
    assert!(
        !own_mask.contains(THIRD_PARTY),
        "a unit is not observed by an inactive third faction"
    );
    assert!(
        world.unit_is_seen_by(own, WATCHER),
        "unit_is_seen_by reports true for observing faction"
    );
    assert!(
        !world.unit_is_seen_by(own, STRANGER),
        "unit_is_seen_by reports false for unobserving faction"
    );
    assert!(
        world.sees_unit(WATCHER, own),
        "sees_unit reports true for observing faction"
    );
    assert!(
        !world.sees_unit(STRANGER, own),
        "sees_unit reports false for unobserving faction"
    );

    // The rival's unit observes its own tile.
    let rival_mask = world.unit_mask(rival);
    assert!(
        rival_mask.contains(STRANGER),
        "the rival unit is observed by the rival faction"
    );
    assert!(
        !rival_mask.contains(WATCHER),
        "the distant rival unit is not observed by the watcher faction"
    );
}

#[test]
fn unit_mask_records_multiple_factions_when_observed_by_both() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let away = ground_near(&world, Axial::new(80, 80), 8);

    let own = a_unit_at(&mut world, home, WATCHER);
    let rival = a_unit_at(&mut world, away, STRANGER);
    world.step(1).expect("the step runs");

    // Initially neither sees the other.
    assert!(!world.unit_is_seen_by(rival, WATCHER));
    assert!(!world.unit_is_seen_by(own, STRANGER));

    // Move rival into the watcher's sight.
    let close = watched_empty_ground(&world, home);
    world
        .place_soldier(rival, close)
        .expect("placement succeeds");
    world.step(1).expect("the step runs");

    // Now both units are in shared sight.
    let own_mask = world.unit_mask(own);
    let rival_mask = world.unit_mask(rival);

    assert!(own_mask.contains(WATCHER), "own unit is seen by watcher");
    assert!(
        own_mask.contains(STRANGER),
        "own unit is seen by stranger who walked close"
    );
    assert!(
        rival_mask.contains(WATCHER),
        "rival unit is seen by watcher"
    );
    assert!(
        rival_mask.contains(STRANGER),
        "rival unit is seen by stranger"
    );

    // Both units appear in units_seen_by for both factions.
    let watcher_seen = world.units_seen_by(WATCHER);
    assert_eq!(watcher_seen.len(), 2, "watcher sees both units");

    let stranger_seen = world.units_seen_by(STRANGER);
    assert_eq!(stranger_seen.len(), 2, "stranger sees both units");

    // Move rival away again.
    world
        .place_soldier(rival, away)
        .expect("placement succeeds");
    world.step(1).expect("the step runs");

    let own_mask_after = world.unit_mask(own);
    let rival_mask_after = world.unit_mask(rival);

    assert!(
        !own_mask_after.contains(STRANGER),
        "own unit loses stranger sight after stranger leaves"
    );
    assert!(
        !rival_mask_after.contains(WATCHER),
        "rival unit loses watcher sight after moving away"
    );
    assert_eq!(world.units_seen_by(WATCHER).len(), 1);
    assert_eq!(world.units_seen_by(STRANGER).len(), 1);
}

#[test]
fn unit_mask_matches_sees_now_for_every_live_unit() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let away = ground_near(&world, Axial::new(80, 80), 8);

    let own = a_unit_at(&mut world, home, WATCHER);
    let rival = a_unit_at(&mut world, away, STRANGER);
    world.step(1).expect("the step runs");

    let obs = world.observation();
    for entity in [own, rival] {
        let tile = world.soldiers().tile(entity).unwrap();
        for faction_idx in 0..world.faction_count() {
            let faction = FactionId(faction_idx);
            let from_mask = obs.unit_is_seen_by(entity, faction);
            let from_layer = obs.sees_now(faction, tile);
            assert_eq!(
                from_mask, from_layer,
                "unit_is_seen_by must match sees_now for entity {entity:?} and faction {faction:?}"
            );
        }
    }
}

#[test]
fn dead_or_despawned_unit_returns_empty_mask() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let own = a_unit_at(&mut world, home, WATCHER);
    world.step(1).expect("the step runs");

    assert!(world.unit_is_seen_by(own, WATCHER));

    // Despawn the unit.
    assert!(world.despawn_soldier(own));

    // World-level queries refuse dead unit.
    assert_eq!(world.unit_mask(own), FactionMask::EMPTY);
    assert!(!world.unit_is_seen_by(own, WATCHER));
    assert!(!world.sees_unit(WATCHER, own));
}

#[test]
fn unit_mask_refuses_unseen_faction_and_fails_if_corrupted() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);
    let own = a_unit_at(&mut world, home, WATCHER);
    world.step(1).expect("the step runs");

    let mask = world.unit_mask(own);
    // Prove that an unreached faction is absent.
    assert!(!mask.contains(STRANGER));

    // A corrupted mask that falsely sets STRANGER must be distinguishable.
    let corrupted = mask.with(STRANGER);
    assert!(
        corrupted.contains(STRANGER),
        "corrupted mask contains stranger"
    );
    assert_ne!(mask, corrupted, "real mask differs from corrupted mask");
}
