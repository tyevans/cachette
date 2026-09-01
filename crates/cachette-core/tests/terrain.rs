//! The terrain field.
//!
//! The test goes through the public crate API. It reaches into no internal
//! module.[^1]
//!
//! Two families of test live here, and they answer different questions.
//!
//! The first family asks whether the field repeats. It reads the world in
//! several orders and at several thread counts, and compares the results.
//!
//! The second family asks what the field depends on. A determinism test
//! cannot tell a correct field from a consistently wrong one, so each field
//! of the draw key gets its own test: change the field, and the output must
//! change.[^2] The row component of the lattice address is the one that a
//! perturbed build drops, which is how these tests are proved able to
//! fail.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`

use std::collections::BTreeMap;

use cachette_core::rng;
use cachette_core::terrain::{admits_a_unit, Terrain, TileKind, KIND_COUNT, TERRAIN_FRAME};
use cachette_core::types::Fix32;
use cachette_core::{Axial, Grid, TileIdx, World, WorldConfig};

/// The extent that most tests read. It is wide enough to hold several
/// lattice cells of the coarsest octave, so a field that varies only inside
/// one cell would not pass.
const WIDTH: u32 = 192;
/// The number of rows of that extent.
const HEIGHT: u32 = 192;
/// The seed that most tests read.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// Builds the terrain under test.
fn terrain(seed: u64) -> Terrain {
    let grid = Grid::new(WIDTH, HEIGHT).expect("the extent must describe a grid");
    Terrain::new(seed, grid)
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

/// Reads the kind of every address in the given order, keyed by address.
fn kinds(field: Terrain, order: &[Axial]) -> BTreeMap<Axial, TileKind> {
    order
        .iter()
        .map(|address| {
            (
                *address,
                field
                    .kind(*address)
                    .expect("the address is inside the world"),
            )
        })
        .collect()
}

/// Reorders the addresses by a stride that is coprime with the tile count.
///
/// The result visits every address exactly once, in an order that shares no
/// run with the row-major order.
fn scattered(order: &[Axial]) -> Vec<Axial> {
    let count = order.len();
    let stride = 7919usize;
    assert_ne!(count % stride, 0, "the stride must not divide the count");
    (0..count).map(|i| order[(i * stride) % count]).collect()
}

#[test]
fn the_same_seed_gives_the_same_kind_whatever_the_visit_order() {
    let field = terrain(SEED);
    let row_major = addresses();
    let scatter = scattered(&row_major);
    let mut reversed = row_major.clone();
    reversed.reverse();

    let expected = kinds(field, &row_major);
    assert_eq!(kinds(field, &scatter), expected);
    assert_eq!(kinds(field, &reversed), expected);
}

#[test]
fn the_same_seed_gives_the_same_kind_at_every_thread_count() {
    let field = terrain(SEED);
    let all = addresses();
    let expected = kinds(field, &all);

    for threads in [1usize, 2, 12] {
        let chunk = all.len().div_ceil(threads).max(1);
        let joined: BTreeMap<Axial, TileKind> = std::thread::scope(|scope| {
            let handles: Vec<_> = all
                .chunks(chunk)
                .map(|part| scope.spawn(move || kinds(field, part)))
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| handle.join().expect("a reader thread must not panic"))
                .collect()
        });
        assert_eq!(joined, expected, "the field changed at {threads} threads");
    }
}

#[test]
fn a_different_seed_gives_a_different_world() {
    // The seed is a field of the draw key. Change it, and the output must
    // change.
    let first = kinds(terrain(SEED), &addresses());
    let second = kinds(terrain(SEED ^ 1), &addresses());
    assert_ne!(first, second, "the seed does not reach the terrain");
}

#[test]
fn the_column_of_an_address_reaches_the_field() {
    // The column is half of the lattice node key. Hold the row and vary the
    // column: the height must not be constant.
    let field = terrain(SEED);
    let row = HEIGHT as i32 / 2;
    let mut seen: Vec<Fix32> = (0..WIDTH as i32)
        .map(|q| {
            field
                .height(Axial::new(q, row))
                .expect("the address is inside the world")
        })
        .collect();
    seen.dedup();
    assert!(
        seen.len() > 1,
        "the height is constant along a row, so the column does not reach the key"
    );
}

#[test]
fn the_row_of_an_address_reaches_the_field() {
    // The row is the other half of the lattice node key. This is the field
    // that the perturbed build drops.
    let field = terrain(SEED);
    let column = WIDTH as i32 / 2;
    let mut seen: Vec<Fix32> = (0..HEIGHT as i32)
        .map(|r| {
            field
                .height(Axial::new(column, r))
                .expect("the address is inside the world")
        })
        .collect();
    seen.dedup();
    assert!(
        seen.len() > 1,
        "the height is constant down a column, so the row does not reach the key"
    );
}

#[test]
fn the_two_fields_take_different_draw_indices() {
    // The draw index is a field of the key. The height field and the
    // moisture field differ only in it, so a field that ignored the draw
    // index would give one value for both.
    let field = terrain(SEED);
    let same = addresses()
        .into_iter()
        .filter(|address| {
            let tile = field.tile(*address).expect("the address is inside");
            tile.height == tile.moisture
        })
        .count();
    assert!(
        same * 2 < (WIDTH * HEIGHT) as usize,
        "the height and the moisture agree on most tiles, so the draw index \
         does not reach the key"
    );
}

#[test]
fn the_terrain_owns_its_system_identifier() {
    // The system is the fourth field of the draw key, and it is the one
    // field a terrain test cannot vary through the terrain interface,
    // because the generator names its own system. Two statements together
    // cover it.
    //
    // First, the identifier is distinct. Two systems that share one draw
    // the same value from the same frame, entity and draw index, so their
    // fields would correlate.
    let identifiers = [
        rng::SYSTEM_TILE_STUB,
        rng::SYSTEM_SOLDIER_MOVE,
        rng::SYSTEM_TERRAIN,
    ];
    for (first, left) in identifiers.iter().enumerate() {
        for (second, right) in identifiers.iter().enumerate() {
            if first != second {
                assert_ne!(left, right, "two systems share one identifier");
            }
        }
    }

    // Second, the slot reaches the value. A generator that ignored the
    // system field would give one value for all three, and the distinctness
    // above would buy nothing.
    let values: Vec<u64> = identifiers
        .iter()
        .map(|system| rng::draw(SEED, *system, TERRAIN_FRAME, 12_345, 0))
        .collect();
    assert_ne!(values[0], values[2]);
    assert_ne!(values[1], values[2]);
}

#[test]
fn the_ground_reaches_the_state_hash() {
    // The ground is part of the world, so the whole-world hash covers it.
    // Two worlds whose ground differs must hash differently, and the only
    // thing that differs here is the seed, which reaches the tile columns
    // too. The stronger statement is the golden file, which pins the
    // generator itself.
    let config = WorldConfig {
        width: 24,
        height: 24,
        seed: SEED,
        faction_count: 2,
    };
    let world = World::new(config).expect("the extent must describe a world");
    let mut other = config;
    other.seed = SEED ^ 0xff;
    let other = World::new(other).expect("the extent must describe a world");
    assert_ne!(world.state_hash(), other.state_hash());
    // The hash of one world is stable across calls, because the ground is
    // computed the same way every time.
    assert_eq!(world.state_hash(), world.state_hash());
}

#[test]
fn the_frame_is_pinned_so_the_terrain_does_not_move() {
    // The frame slot of the key holds a constant on purpose. A world that
    // steps must read the same terrain afterwards.
    assert_eq!(TERRAIN_FRAME, 0);
    let mut world = World::new(WorldConfig {
        width: 48,
        height: 48,
        seed: SEED,
        faction_count: 2,
    })
    .expect("the extent must describe a world");
    let before: Vec<TileKind> = (0..world.tile_count())
        .map(|i| {
            world
                .terrain()
                .tile_at(TileIdx(i as u32))
                .expect("the index is inside the world")
                .kind
        })
        .collect();
    for _ in 0..8 {
        world.step(3).expect("the step must run");
    }
    let after: Vec<TileKind> = (0..world.tile_count())
        .map(|i| {
            world
                .terrain()
                .tile_at(TileIdx(i as u32))
                .expect("the index is inside the world")
                .kind
        })
        .collect();
    assert_eq!(before, after);
}

#[test]
fn every_height_is_inside_the_unit_range() {
    let field = terrain(SEED);
    for address in addresses() {
        let tile = field.tile(address).expect("the address is inside");
        assert!(tile.height >= Fix32::ZERO && tile.height < Fix32::ONE);
        assert!(tile.moisture >= Fix32::ZERO && tile.moisture < Fix32::ONE);
    }
}

#[test]
fn the_thresholds_give_every_kind_a_share_of_the_world() {
    // A determinism test passes over a field that is one kind everywhere.
    // This test does not.
    let field = terrain(SEED);
    let mut counts = [0usize; KIND_COUNT];
    for address in addresses() {
        counts[field.kind(address).expect("inside").to_u8() as usize] += 1;
    }
    for (kind, count) in counts.iter().enumerate() {
        assert!(*count > 0, "kind {kind} occurs on no tile");
    }
    assert!(
        counts[TileKind::Water.to_u8() as usize] > 0
            && counts[TileKind::Mountain.to_u8() as usize] > 0
    );
}

#[test]
fn the_field_is_smooth_between_neighbours() {
    // Value noise with a smooth weight gives a field where a neighbour is
    // close. White noise would not. This is what separates terrain from a
    // per-tile draw.
    let field = terrain(SEED);
    let grid = field.grid();
    let mut worst = 0i32;
    for address in addresses() {
        let here = field.height(address).expect("inside").0;
        for direction in 0..6 {
            if let Some(next) = grid.neighbour(address, direction) {
                let there = field.height(next).expect("inside").0;
                worst = worst.max((here - there).abs());
            }
        }
    }
    // One eighth of the unit range. A white-noise field reaches the whole
    // range between neighbours and fails this.
    assert!(
        worst < Fix32::ONE.0 / 8,
        "neighbouring heights differ by {worst}, so the field is not smooth"
    );
}

#[test]
fn an_address_outside_the_world_has_no_terrain() {
    let field = terrain(SEED);
    assert!(field.tile(Axial::new(-1, 0)).is_none());
    assert!(field.tile(Axial::new(0, -1)).is_none());
    assert!(field.tile(Axial::new(WIDTH as i32, 0)).is_none());
    assert!(field.tile(Axial::new(0, HEIGHT as i32)).is_none());
    assert!(field.tile_at(TileIdx(WIDTH * HEIGHT)).is_none());
}

#[test]
fn the_index_and_the_address_name_one_tile() {
    let field = terrain(SEED);
    let grid = field.grid();
    for address in addresses() {
        let index = grid.index_of(address).expect("inside");
        assert_eq!(field.tile_at(index), field.tile(address));
    }
}

#[test]
fn passability_is_the_capacity_of_the_ground() {
    assert_eq!(TileKind::ALL.len(), KIND_COUNT);
    for kind in TileKind::ALL {
        assert_eq!(
            kind.is_passable(),
            kind.capacity() > 0,
            "the kind {kind:?} answers passability and capacity differently"
        );
    }
}

#[test]
fn ground_that_holds_nobody_admits_nobody() {
    assert!(!admits_a_unit(0));
    assert!(admits_a_unit(1));
}

#[test]
fn at_least_one_kind_refuses_a_unit_and_at_least_one_admits_one() {
    assert!(TileKind::ALL.iter().any(|kind| !kind.is_passable()));
    assert!(TileKind::ALL.iter().any(|kind| kind.is_passable()));
}

#[test]
fn the_world_and_the_terrain_agree() {
    let config = WorldConfig {
        width: 32,
        height: 32,
        seed: SEED,
        faction_count: 2,
    };
    let world = World::new(config).expect("the extent must describe a world");
    assert!(world.check_invariants());
    assert_eq!(world.terrain().seed(), config.seed);
    let address = Axial::new(5, 7);
    assert_eq!(
        world.tile_kind(address),
        world.tile_terrain(address).map(|tile| tile.kind)
    );
    // Two worlds from one seed hold one terrain.
    let other = World::new(config).expect("the extent must describe a world");
    assert_eq!(world.terrain(), other.terrain());
}
