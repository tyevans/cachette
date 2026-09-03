//! The census of one window of the world.
//!
//! A caller outside this crate reads one address at a time. The census runs
//! the loop inside the engine and answers once, so that the control plane
//! never runs one.[^1] These tests go through the public interface of the
//! crate.[^2]
//!
//! Every world here is wider than the coarsest lattice spacing of the terrain
//! generator. A narrower world sits inside one lattice cell and holds one kind
//! of ground, and a census of it would measure the fixture rather than the
//! census.[^3]
//!
//! # References
//!
//! [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
//! [^2]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^3]: Findings register, FND-054. `docs/FINDINGS.md`

use cachette_core::census::{census, CensusError, RADIUS_CEILING};
use cachette_core::terrain::KIND_COUNT;
use cachette_core::{Axial, FactionId, World, WorldConfig};

/// The extent of the fixture.
///
/// The coarsest lattice of the generator spans sixty-four tiles, so this world
/// holds three lattice cells along each axis and therefore holds more than one
/// kind of ground.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const EXTENT: u32 = 192;

fn world() -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: 0x0123_4567_89ab_cdef,
        faction_count: 4,
        unit_capacity: 4096,
    })
    .expect("the extent describes a world")
}

/// Counts a rectangle of addresses one at a time.
///
/// **This is the loop the census exists to replace.** The test runs it so that
/// the census has something to be wrong against. A census that agreed with
/// itself would prove nothing.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 1. `.claude/rules/testing.md`
fn by_hand(world: &World, first: Axial, last: Axial) -> (i64, [i64; KIND_COUNT], i64) {
    let mut tiles = 0;
    let mut by_kind = [0i64; KIND_COUNT];
    let mut open = 0;
    for r in first.r..=last.r {
        for q in first.q..=last.q {
            let address = Axial::new(q, r);
            let kind = world
                .tile_kind(address)
                .expect("the address is in the world");
            tiles += 1;
            by_kind[kind.to_u8() as usize] += 1;
            if world.admits_a_unit(address) {
                open += 1;
            }
        }
    }
    (tiles, by_kind, open)
}

#[test]
fn a_census_equals_the_addresses_it_read() {
    let world = world();
    let counted = census(&world, Axial::new(96, 96), 5).expect("the window is in the world");
    let (tiles, by_kind, open) = by_hand(&world, counted.first(), counted.last());

    assert_eq!(counted.tiles(), tiles);
    assert_eq!(counted.by_kind(), &by_kind);
    assert_eq!(counted.open_tiles(), open);
    assert_eq!(counted.tiles(), by_kind.iter().sum::<i64>());
}

#[test]
fn a_census_reads_the_window_the_caller_named() {
    let world = world();
    let counted = census(&world, Axial::new(96, 96), 3).expect("the window is in the world");

    assert_eq!(counted.first(), Axial::new(93, 93));
    assert_eq!(counted.last(), Axial::new(99, 99));
    assert_eq!(counted.tiles(), 7 * 7);
}

#[test]
fn a_census_clips_the_window_to_the_world() {
    let world = world();
    let counted = census(&world, Axial::new(0, 0), 3).expect("the window meets the world");

    assert_eq!(counted.first(), Axial::new(0, 0));
    assert_eq!(counted.last(), Axial::new(3, 3));
    assert_eq!(counted.tiles(), 4 * 4);
}

#[test]
fn a_census_refuses_a_window_the_world_does_not_reach() {
    let world = world();
    let far = Axial::new(-1000, -1000);

    assert_eq!(
        census(&world, far, 4),
        Err(CensusError::WindowOutsideWorld(far))
    );
}

#[test]
fn a_census_refuses_a_radius_above_the_ceiling() {
    let world = world();

    assert_eq!(
        census(&world, Axial::new(96, 96), RADIUS_CEILING + 1),
        Err(CensusError::RadiusAboveCeiling {
            asked: RADIUS_CEILING + 1,
            ceiling: RADIUS_CEILING,
        })
    );
}

#[test]
fn a_census_counts_the_units_that_stand_in_the_window() {
    let mut world = world();
    let places: Vec<Axial> = (0..3)
        .map(|step| Axial::new(96 + step, 96))
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    assert!(
        places.len() > 1,
        "the fixture needs more than one open tile in the window"
    );
    for address in &places {
        world
            .spawn_soldier(*address, FactionId(0))
            .expect("the ground admits a unit");
    }
    // The bridge is derived and rebuilds at the barrier, so a census before
    // the rebuild reads a bridge that predates the spawn.
    assert!(matches!(
        census(&world, Axial::new(96, 96), 2),
        Err(CensusError::Bridge(_))
    ));
    world.rebuild_bridge(1).expect("the rebuild runs");

    let counted = census(&world, Axial::new(96, 96), 2).expect("the window is in the world");

    assert_eq!(counted.units(), places.len() as i64);
    assert_eq!(counted.crowd_worst(), 1);
    assert_eq!(counted.crowded_most(), Some(places[0]));
}

#[test]
fn a_census_names_the_tile_that_holds_the_most() {
    let mut world = world();
    let crowded = Axial::new(96, 96);
    assert!(
        world.admits_a_unit(crowded),
        "the fixture needs open ground"
    );
    let room = world
        .tile_capacity(crowded)
        .expect("the address is in the world");
    for _ in 0..room {
        world
            .spawn_soldier(crowded, FactionId(0))
            .expect("a spawn may over-fill a tile");
    }
    world.rebuild_bridge(1).expect("the rebuild runs");

    let counted = census(&world, crowded, 2).expect("the window is in the world");

    assert_eq!(counted.crowd_worst(), room);
    assert_eq!(counted.crowded_most(), Some(crowded));
    assert_eq!(counted.tiles_at_capacity(), 1);
}

#[test]
fn an_empty_tile_is_never_at_its_capacity() {
    // Water admits nobody, so a rule that compared a count against a capacity
    // without asking for a unit first would call every tile of open water
    // full. The drawing pass takes the same rule, and the two counts would
    // otherwise disagree over every coast.
    let world = world();
    let counted = census(&world, Axial::new(96, 96), 8).expect("the window is in the world");

    assert_eq!(counted.units(), 0);
    assert_eq!(counted.tiles_at_capacity(), 0);
}
