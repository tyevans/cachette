//! The hazards that stand over one tile.
//!
//! The engine holds two hazards and no others. A tile burns, and a storm
//! stands over it. The fire field holds the first and the weather field holds
//! the second. Nothing here invents a third.
//!
//! **The fixture reaches each hazard on its own, and both at once.** A fixture
//! that only lit a fire would pass a reader that read the fire alone, and a
//! fixture that only raised a storm would pass one that read the storm
//! alone.[^1]
//!
//! The tests drive the public verbs and then read the world.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 5. `.agents/rules/testing.md`

use cachette_core::{Axial, CycloneSetting, TileKind, World, WorldConfig};

/// The extent of the fixture world.
///
/// The world is wide enough to hold a weather cell that no storm reaches, so
/// the fixture reaches the case of a tile under nothing.
const EDGE: u32 = 64;

/// Builds the fixture world.
fn world() -> World {
    World::new(WorldConfig {
        width: EDGE,
        height: EDGE,
        seed: 11,
        faction_count: 2,
        unit_capacity: 16,
    })
    .expect("the extent must describe a world")
}

/// Returns the passable addresses of a world, in tile index order.
fn dry_ground(world: &World) -> Vec<Axial> {
    let mut dry = Vec::new();
    for row in 0..world.grid().height() {
        for column in 0..world.grid().width() {
            let here = Axial::new(column as i32, row as i32);
            if world.tile_kind(here).is_some_and(TileKind::is_passable) {
                dry.push(here);
            }
        }
    }
    dry
}

#[test]
fn a_tile_under_nothing_reports_no_hazard() {
    let mut world = world();
    world.step(1).expect("the step must run");
    let quiet = dry_ground(&world);
    assert!(
        quiet
            .iter()
            .all(|here| world.tile_under_a_hazard(*here) == Some(false)),
        "a world with no fire and no storm must report no hazard anywhere"
    );
}

#[test]
fn a_burning_tile_reports_a_hazard() {
    let mut world = world();
    world.step(1).expect("the step must run");
    let dry = dry_ground(&world);
    let alight = *dry.first().expect("the world holds passable ground");
    let tile = world
        .grid()
        .index_of(alight)
        .expect("the address is inside the world");
    assert!(world.ignite(tile), "the verb refused the ignition");
    assert_eq!(world.tile_is_burning(alight), Some(true));
    assert_eq!(
        world.tile_under_a_storm(alight),
        Some(false),
        "the fixture must reach a fire with no storm over it"
    );
    assert_eq!(world.tile_under_a_hazard(alight), Some(true));
}

/// Raises a small storm over the fixture world and steps it once.
///
/// **The storm reaches part of the world and not the whole of it.** The
/// settings run from one cell to several, and the broad setting covers every
/// cell of a world this small, so a fixture that took it would never reach a
/// tile outside the storm.[^1]
///
/// The eye drifts on the tick, so the caller reads which tiles the storm
/// covers rather than assuming the tile it was raised over.
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
fn a_world_under_one_storm() -> (World, Vec<Axial>, Vec<Axial>) {
    let mut world = world();
    world.step(1).expect("the step must run");
    let dry = dry_ground(&world);
    let eye = *dry.first().expect("the world holds passable ground");
    world
        .raise_cyclone(eye, CycloneSetting::SEVERE)
        .expect("the place lies inside the world");
    world.step(1).expect("the step must run");
    let (under, clear): (Vec<Axial>, Vec<Axial>) = dry
        .into_iter()
        .partition(|here| world.tile_under_a_storm(*here) == Some(true));
    (world, under, clear)
}

#[test]
fn a_tile_under_a_storm_reports_a_hazard() {
    let (world, under, clear) = a_world_under_one_storm();
    assert!(
        !under.is_empty(),
        "the fixture must reach a tile the storm carries"
    );
    assert!(
        !clear.is_empty(),
        "the fixture must reach a tile the storm does not carry"
    );
    for here in &under {
        assert_eq!(
            world.tile_is_burning(*here),
            Some(false),
            "the fixture must reach a storm with no fire under it"
        );
        assert_eq!(world.tile_under_a_hazard(*here), Some(true));
    }
    for here in &clear {
        assert_eq!(world.tile_under_a_hazard(*here), Some(false));
    }
}

#[test]
fn a_tile_under_both_hazards_reports_one_hazard() {
    let (mut world, under, _) = a_world_under_one_storm();
    let place = *under
        .first()
        .expect("the fixture must reach a tile the storm carries");
    let tile = world
        .grid()
        .index_of(place)
        .expect("the address is inside the world");
    assert!(world.ignite(tile), "the verb refused the ignition");
    assert_eq!(world.tile_is_burning(place), Some(true));
    assert_eq!(world.tile_under_a_storm(place), Some(true));
    assert_eq!(world.tile_under_a_hazard(place), Some(true));
}

#[test]
fn an_address_outside_the_world_reports_nothing() {
    let world = world();
    let outside = Axial::new(EDGE as i32, EDGE as i32);
    assert_eq!(world.tile_under_a_storm(outside), None);
    assert_eq!(world.tile_under_a_hazard(outside), None);
}
