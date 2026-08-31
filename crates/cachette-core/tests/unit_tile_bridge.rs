//! The unit-to-tile bridge.
//!
//! A soldier holds the tile it stands on. The bridge holds the other
//! direction, and it is wholly derived from the soldier columns.[^1]
//!
//! The bridge rebuilds at the frame barrier by a sort on a block-major key,
//! and the tie-break is the whole identity.[^2] The sort takes a key vector
//! of exact integer fields and no comparison function.[^3]
//!
//! A per-tile query reads the block range, then searches inside it.[^4] An
//! empty block carries a clear bit in the occupancy bitplane.[^5]
//!
//! The tests see only the public crate API.[^6]
//!
//! # References
//!
//! [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^3]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
//! [^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D5. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
//! [^6]: Testing policy. `docs/TESTING.md`

use cachette_core::bridge::BLOCK_BITS_CEILING;
use cachette_core::sort::{self, SortKey};
use cachette_core::{
    Axial, BlockLayout, BridgeError, Entity, FactionId, Grid, SoldierArena, UnitTileBridge, World,
    WorldConfig,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// The extent of the world that the tests place soldiers on.
///
/// The width is not a multiple of the block edge, so the last block column
/// is short. A world that divided evenly would hide a defect in the block
/// derivation.
const WIDTH: u32 = 10;
/// The extent of the world that the tests place soldiers on.
const HEIGHT: u32 = 9;
/// The block edge exponent that the tests partition by. Four tiles.
const BLOCK_BITS: u32 = 2;
/// The thread counts that every rebuild runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// Builds the world shape that the tests use.
fn grid() -> Grid {
    Grid::new(WIDTH, HEIGHT).expect("a small extent describes a grid")
}

/// Builds an empty bridge over the test world.
fn bridge() -> UnitTileBridge {
    let layout = BlockLayout::new(grid(), BLOCK_BITS).expect("the exponent is inside the ceiling");
    UnitTileBridge::new(layout)
}

/// Returns the address of a tile ordinal inside the test world.
fn address(ordinal: u32) -> Axial {
    let inside = ordinal % (WIDTH * HEIGHT);
    Axial::new((inside % WIDTH) as i32, (inside / WIDTH) as i32)
}

/// Returns the soldiers on one tile by scanning the whole population.
///
/// This is the answer that the bridge must give without the scan. The order
/// is the identity order, which is the order the bridge holds.[^1]
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
fn by_scan(arena: &SoldierArena, place: Axial) -> Vec<Entity> {
    let mut found: Vec<Entity> = arena
        .iter()
        .filter(|soldier| arena.address(*soldier) == Some(place))
        .collect();
    found.sort_by_key(|soldier| soldier.to_bits());
    found
}

/// Fills an arena with a fixed pattern and returns it.
///
/// Many soldiers share one tile, because the tie-break is the only thing
/// that fixes their order. The pattern despawns inside the loop, so a later
/// spawn takes a freed slot at a higher generation. Two soldiers on one tile
/// therefore rank differently by slot index and by whole identity, and only
/// the whole identity is the tie-break.[^1]
///
/// # References
///
/// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
fn populate(count: u32) -> SoldierArena {
    let mut arena = SoldierArena::new(grid());
    let mut live: Vec<Entity> = Vec::new();
    for ordinal in 0..count {
        // The step of seven visits many tiles, and the remainder of three
        // sends every third soldier to one shared tile.
        let place = if ordinal % 3 == 0 {
            Axial::new(3, 4)
        } else {
            address(ordinal * 7)
        };
        let soldier = arena
            .spawn(place, FactionId((ordinal % 5) as u16))
            .expect("the address and the faction must be valid");
        live.push(soldier);
        if ordinal % 5 == 4 && live.len() > 2 {
            // Free a slot now, so the next spawn reuses it at generation two.
            let victim = live.remove(live.len() / 3);
            assert!(arena.despawn(victim));
        }
    }
    arena
}

#[test]
fn a_bridge_refuses_an_arena_it_was_not_built_from() {
    // The revision counts changes; it does not name the arena. Two arenas of
    // one extent, each holding one soldier on a different tile, both sit at
    // revision one. A guard that compared only the count would pass, and the
    // bridge would answer questions about an arena it never read.
    let grid = grid();
    let mut first = SoldierArena::new(grid);
    let mut second = SoldierArena::new(grid);
    first
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the spawn must succeed");
    second
        .spawn(Axial::new(4, 4), FactionId(0))
        .expect("the spawn must succeed");
    assert_eq!(first.revision(), second.revision());

    let layout = BlockLayout::new(grid, BLOCK_BITS).expect("the exponent is inside the ceiling");
    let mut bridge = UnitTileBridge::new(layout);
    bridge.rebuild(&first, 1).expect("the rebuild must succeed");

    assert!(bridge.on_tile(&first, Axial::new(0, 0)).is_ok());
    assert_eq!(
        bridge.on_tile(&second, Axial::new(4, 4)),
        Err(BridgeError::WrongArena),
    );
    assert_eq!(
        bridge.count_on_tile(&second, Axial::new(4, 4)),
        Err(BridgeError::WrongArena),
    );
}

#[test]
fn a_bridge_that_was_never_built_refuses_every_read() {
    let arena = SoldierArena::new(grid());
    let bridge = bridge();
    assert_eq!(
        bridge.on_tile(&arena, Axial::new(0, 0)),
        Err(BridgeError::NeverBuilt)
    );
    assert_eq!(
        bridge.check_invariants(&arena),
        Err(BridgeError::NeverBuilt)
    );
}

#[test]
fn a_read_after_a_move_is_refused_rather_than_answered() {
    // The stale read is the mistake this interface exists to stop. A comment
    // that says "rebuild first" would be the defect shape the project
    // records.[^1]
    //
    // [^1]: Findings register, FND-040. `docs/FINDINGS.md`
    let mut arena = SoldierArena::new(grid());
    let soldier = arena
        .spawn(Axial::new(1, 1), FactionId(0))
        .expect("the spawn must succeed");
    let mut bridge = bridge();
    bridge.rebuild(&arena, 1).expect("the rebuild must succeed");
    assert_eq!(bridge.on_tile(&arena, Axial::new(1, 1)), Ok(&[soldier][..]));

    assert_eq!(arena.place(soldier, Axial::new(9, 8)), Ok(true));
    let stale = bridge.on_tile(&arena, Axial::new(1, 1));
    assert!(
        matches!(stale, Err(BridgeError::Stale { .. })),
        "a read after a move must be refused, and it gave {stale:?}"
    );
    assert!(matches!(
        bridge.on_tile(&arena, Axial::new(9, 8)),
        Err(BridgeError::Stale { .. })
    ));
    assert!(matches!(
        bridge.check_invariants(&arena),
        Err(BridgeError::Stale { .. })
    ));

    bridge.rebuild(&arena, 1).expect("the rebuild must succeed");
    assert_eq!(bridge.on_tile(&arena, Axial::new(1, 1)), Ok(&[][..]));
    assert_eq!(bridge.on_tile(&arena, Axial::new(9, 8)), Ok(&[soldier][..]));
}

#[test]
fn a_spawn_and_a_despawn_each_make_the_bridge_stale() {
    let mut arena = SoldierArena::new(grid());
    let mut bridge = bridge();
    bridge.rebuild(&arena, 1).expect("the rebuild must succeed");
    let soldier = arena
        .spawn(Axial::new(2, 2), FactionId(0))
        .expect("the spawn must succeed");
    assert!(matches!(
        bridge.on_tile(&arena, Axial::new(2, 2)),
        Err(BridgeError::Stale { .. })
    ));

    bridge.rebuild(&arena, 1).expect("the rebuild must succeed");
    assert!(arena.despawn(soldier));
    assert!(matches!(
        bridge.on_tile(&arena, Axial::new(2, 2)),
        Err(BridgeError::Stale { .. })
    ));
}

#[test]
fn a_bridge_over_another_world_is_refused() {
    let arena = SoldierArena::new(Grid::new(4, 4).expect("a small extent describes a grid"));
    let mut bridge = bridge();
    assert_eq!(bridge.rebuild(&arena, 1), Err(BridgeError::GridMismatch));
    assert_eq!(
        bridge.on_tile(&arena, Axial::new(0, 0)),
        Err(BridgeError::GridMismatch)
    );
}

#[test]
fn an_address_outside_the_world_is_refused() {
    let arena = SoldierArena::new(grid());
    let mut bridge = bridge();
    bridge.rebuild(&arena, 1).expect("the rebuild must succeed");
    let outside = Axial::new(WIDTH as i32, 0);
    assert_eq!(
        bridge.on_tile(&arena, outside),
        Err(BridgeError::AddressOutsideWorld(outside))
    );
}

#[test]
fn a_block_edge_above_the_ceiling_is_refused() {
    let bits = BLOCK_BITS_CEILING + 1;
    assert_eq!(
        BlockLayout::new(grid(), bits),
        Err(BridgeError::BlockBitsAboveCeiling(bits))
    );
}

#[test]
fn a_rebuild_at_zero_threads_is_refused() {
    let arena = SoldierArena::new(grid());
    let mut bridge = bridge();
    assert!(bridge.rebuild(&arena, 0).is_err());
}

#[test]
fn the_keys_of_one_block_hold_one_run_of_the_key_space() {
    // The block range is a start and a length rather than a list of runs,
    // and that holds only when the key is block-major. No key of another
    // block falls between two keys of one block.[^1]
    //
    // [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    let layout = BlockLayout::new(grid(), BLOCK_BITS).expect("the exponent is inside the ceiling");
    let world = grid();
    let mut keys: Vec<(u64, u32)> = Vec::new();
    for index in 0..world.tile_count() {
        let key = layout
            .key_of(cachette_core::TileIdx(index))
            .expect("every tile has a key");
        keys.push((key, layout.block_of_key(key)));
    }
    keys.sort_unstable();

    // Every tile has its own key, and the blocks come in whole runs.
    for pair in keys.windows(2) {
        assert_ne!(pair[0].0, pair[1].0, "two tiles share one key");
    }
    let mut seen = vec![false; layout.block_count() as usize];
    let mut current = keys[0].1;
    seen[current as usize] = true;
    for (_, block) in &keys[1..] {
        if *block == current {
            continue;
        }
        assert!(
            !seen[*block as usize],
            "block {block} holds more than one run"
        );
        seen[*block as usize] = true;
        current = *block;
    }
    assert!(seen.iter().all(|held| *held), "a block holds no tile");
}

#[test]
fn the_key_ceiling_bounds_every_tile_key_at_every_block_edge() {
    // The bounded order takes the ceiling and derives its pass count from it,
    // so a ceiling below a real key would refuse a legal world.[^1] The
    // exponent runs over a range, because a test at one exponent cannot see a
    // defect in how the exponent enters the derivation.
    //
    // [^1]: ADR-0071, the bridge rebuild orders on one thread, decision D1. `docs/adrs/draft/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
    for block_bits in 0..=6 {
        let layout =
            BlockLayout::new(grid(), block_bits).expect("the exponent is inside the ceiling");
        let ceiling = layout.key_ceiling();
        let mut highest = 0u64;
        for index in 0..grid().tile_count() {
            let key = layout
                .key_of(cachette_core::TileIdx(index))
                .expect("every tile has a key");
            assert!(
                key <= ceiling,
                "the key {key} of a tile lies above the ceiling {ceiling} at exponent {block_bits}"
            );
            highest = highest.max(key);
        }
        assert!(
            ceiling >= highest,
            "the ceiling {ceiling} is below the highest key {highest}"
        );
    }
}

#[test]
fn a_block_aligned_world_reaches_its_key_ceiling_and_still_rebuilds() {
    // The extent of the other tests does not divide by the block edge, so no
    // tile there carries the ceiling itself. A world that divides evenly
    // does, and the rebuild must accept it.
    let world = Grid::new(8, 8).expect("the extent describes a world");
    let layout = BlockLayout::new(world, 2).expect("the exponent is inside the ceiling");
    let highest = (0..world.tile_count())
        .map(|index| {
            layout
                .key_of(cachette_core::TileIdx(index))
                .expect("every tile has a key")
        })
        .max()
        .expect("the world holds a tile");
    assert_eq!(highest, layout.key_ceiling());

    let mut arena = SoldierArena::new(world);
    arena
        .spawn(Axial::new(7, 7), FactionId(0))
        .expect("the spawn must succeed");
    let mut bridge = UnitTileBridge::new(layout);
    bridge
        .rebuild(&arena, 1)
        .expect("a tile at the ceiling must not refuse the rebuild");
    assert_eq!(bridge.len(), 1);
}

#[test]
fn a_world_rebuilds_at_every_block_edge() {
    // The rebuild derives its ceiling from the partition. A rebuild that
    // refuses a legal world would show here and nowhere else, because every
    // other test uses one exponent.
    let arena = populate(40);
    for block_bits in 0..=6 {
        let layout =
            BlockLayout::new(grid(), block_bits).expect("the exponent is inside the ceiling");
        let mut bridge = UnitTileBridge::new(layout);
        bridge
            .rebuild(&arena, 1)
            .expect("the rebuild must succeed at every exponent");
        assert_eq!(bridge.len(), arena.len() as usize);
        for ordinal in 0..(WIDTH * HEIGHT) {
            let place = address(ordinal);
            let held = bridge.on_tile(&arena, place).expect("the bridge is fresh");
            assert_eq!(held, by_scan(&arena, place).as_slice());
        }
    }
}

#[test]
fn an_empty_block_carries_a_clear_bit_and_an_empty_answer() {
    let mut arena = SoldierArena::new(grid());
    arena
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the spawn must succeed");
    let mut bridge = bridge();
    bridge.rebuild(&arena, 1).expect("the rebuild must succeed");
    let layout = bridge.layout();

    let held = layout
        .key_of(grid().index_of(Axial::new(0, 0)).expect("inside"))
        .expect("every tile has a key");
    assert!(bridge.block_is_occupied(layout.block_of_key(held)));

    let far = Axial::new(9, 8);
    let empty = layout
        .key_of(grid().index_of(far).expect("inside"))
        .expect("every tile has a key");
    assert!(!bridge.block_is_occupied(layout.block_of_key(empty)));
    assert_eq!(bridge.on_tile(&arena, far), Ok(&[][..]));
    assert!(!bridge.block_is_occupied(layout.block_count()));
}

#[test]
fn the_world_answers_a_tile_after_the_step_rebuilds_the_bridge() {
    // Drive the engine, then inspect the bridge. A structure that no test
    // reaches through the engine is inert.[^1]
    //
    // [^1]: Findings register, FND-041. `docs/FINDINGS.md`
    let config = WorldConfig {
        width: 40,
        height: 40,
        seed: 11,
        faction_count: 3,
    };
    let mut world = World::new(config).expect("the extent must describe a world");
    let mut expected = Vec::new();
    for ordinal in 0..48u32 {
        let place = Axial::new((ordinal % 7) as i32, (ordinal / 7) as i32);
        let soldier = world
            .spawn_soldier(place, FactionId((ordinal % 3) as u16))
            .expect("the address and the faction must be valid");
        expected.push((soldier, place));
    }

    // The spawn made the bridge stale. The step is what makes it readable.
    assert!(matches!(
        world.soldiers_on(Axial::new(0, 0)),
        Err(BridgeError::Stale { .. })
    ));
    world.step(4).expect("the step must run");
    assert!(world.check_invariants());

    for (soldier, spawn) in &expected {
        // The step moves each soldier to a neighbour, so the bridge must name
        // it on the tile it reached and not on the tile it left. The move is
        // one tile at most.[^1]
        //
        // [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D1, a draft record. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
        let place = world
            .soldiers()
            .address(*soldier)
            .expect("the soldier is alive");
        assert!(spawn.distance(place) <= 1);
        let held = world.soldiers_on(place).expect("the bridge is fresh");
        assert!(
            held.contains(soldier),
            "the bridge must name the soldier on ({}, {})",
            place.q,
            place.r
        );
        assert_eq!(world.soldier_count_on(place), Ok(held.len()));
    }
    assert_eq!(world.soldiers_on(Axial::new(39, 39)), Ok(&[][..]));
    assert_eq!(world.bridge().len(), expected.len());
}

#[test]
fn a_world_that_rebuilds_outside_a_step_answers_again() {
    let mut world = World::new(WorldConfig {
        width: 16,
        height: 16,
        seed: 3,
        faction_count: 2,
    })
    .expect("the extent must describe a world");
    let soldier = world
        .spawn_soldier(Axial::new(5, 5), FactionId(0))
        .expect("the spawn must succeed");
    assert!(world.soldiers_on(Axial::new(5, 5)).is_err());
    world.rebuild_bridge(2).expect("the rebuild must run");
    assert_eq!(world.soldiers_on(Axial::new(5, 5)), Ok(&[soldier][..]));
    assert!(world.rebuild_bridge(0).is_err());
}

proptest! {
    #![proptest_config(ProptestConfig {
        // An integration test has no lib.rs or main.rs above it, so the
        // default source-parallel persistence finds no root and silently
        // disables itself. A failing seed is then never written and never
        // replayed. Name the file, so that a seed which caught a defect runs
        // first on every later run.[^1]
        //
        // [^1]: Findings register, FND-044. `docs/FINDINGS.md`
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/unit_tile_bridge.proptest-regressions"),
        ))),
        ..ProptestConfig::default()
    })]

    /// The bridge holds exactly the live soldiers, each on its own tile.
    ///
    /// The bridge is a second declaration of where a soldier stands. This
    /// property is the one that makes the two agree.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[test]
    fn the_bridge_holds_exactly_the_live_soldiers(count in 0u32..120) {
        let arena = populate(count);
        let mut bridge = bridge();
        bridge.rebuild(&arena, 1).expect("the rebuild must succeed");

        prop_assert!(bridge.check_structure());
        prop_assert_eq!(bridge.check_invariants(&arena), Ok(true));
        prop_assert_eq!(bridge.len(), arena.len() as usize);
        prop_assert_eq!(bridge.is_empty(), arena.is_empty());

        for soldier in arena.iter() {
            let place = arena.address(soldier).expect("a live soldier has an address");
            let held = bridge.on_tile(&arena, place).expect("the bridge is fresh");
            prop_assert!(held.contains(&soldier));
        }
    }

    /// A per-tile query gives the answer that a whole scan gives.
    #[test]
    fn a_per_tile_query_gives_what_a_scan_gives(count in 0u32..90) {
        let arena = populate(count);
        let mut bridge = bridge();
        bridge.rebuild(&arena, 1).expect("the rebuild must succeed");

        let mut total = 0usize;
        for ordinal in 0..(WIDTH * HEIGHT) {
            let place = address(ordinal);
            let held = bridge.on_tile(&arena, place).expect("the bridge is fresh");
            let scanned = by_scan(&arena, place);
            prop_assert_eq!(held, scanned.as_slice());
            total += held.len();
        }
        prop_assert_eq!(total, arena.len() as usize);
    }

    /// The rebuild gives one answer at every thread count.
    ///
    /// The population shares tiles, so the identity tie-break is what fixes
    /// the order inside a tile.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[test]
    fn the_bridge_is_identical_at_every_thread_count(count in 12u32..120) {
        let arena = populate(count);
        let mut shared = 0usize;
        for ordinal in 0..(WIDTH * HEIGHT) {
            shared = shared.max(by_scan(&arena, address(ordinal)).len());
        }
        prop_assert!(shared > 1, "the population must share a tile");

        let mut first = bridge();
        first.rebuild(&arena, THREAD_COUNTS[0]).expect("the rebuild must succeed");
        let expected: Vec<Vec<Entity>> = (0..(WIDTH * HEIGHT))
            .map(|ordinal| {
                first
                    .on_tile(&arena, address(ordinal))
                    .expect("the bridge is fresh")
                    .to_vec()
            })
            .collect();

        for threads in &THREAD_COUNTS[1..] {
            let mut other = bridge();
            other.rebuild(&arena, *threads).expect("the rebuild must succeed");
            prop_assert_eq!(other.check_invariants(&arena), Ok(true));
            for ordinal in 0..(WIDTH * HEIGHT) {
                let held = other
                    .on_tile(&arena, address(ordinal))
                    .expect("the bridge is fresh");
                prop_assert_eq!(held, expected[ordinal as usize].as_slice());
            }
            for block in 0..other.layout().block_count() {
                prop_assert_eq!(other.block_range(block), first.block_range(block));
                prop_assert_eq!(other.block_is_occupied(block), first.block_is_occupied(block));
            }
        }
    }

    /// The rebuild gives what the general key vector sort gives.
    ///
    /// The bridge orders by a radix sort on the bounded tile key.[^1] The
    /// general sort compares a key vector of two fields. The two are separate
    /// algorithms over one definition of the order, so this property holds
    /// them together. It fails when either one drifts.
    ///
    /// The test rebuilds the key vector from the public layout, so it does
    /// not read a private field of the bridge.
    ///
    /// # References
    ///
    /// [^1]: ADR-0071, the bridge rebuild orders on one thread, decision D1. `docs/adrs/draft/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
    #[test]
    fn the_rebuild_gives_what_the_general_sort_gives(count in 0u32..120) {
        let arena = populate(count);
        let mut bridge = bridge();
        bridge.rebuild(&arena, 1).expect("the rebuild must succeed");

        let layout = bridge.layout();
        let units: Vec<Entity> = arena.iter().collect();
        let keys: Vec<SortKey<2>> = units
            .iter()
            .map(|unit| {
                let tile = arena.tile_column()[unit.index() as usize];
                let key = layout.key_of(tile).expect("the tile is inside the world");
                SortKey::new([key, unit.to_bits()])
            })
            .collect();
        let order = sort::order(&keys).expect("the identities are unique");
        let expected: Vec<Entity> = order.iter().map(|index| units[*index as usize]).collect();

        let mut held: Vec<Entity> = Vec::new();
        for block in 0..layout.block_count() {
            if let Some(range) = bridge.block_range(block) {
                let inside = bridge.in_block(&arena, block).expect("the bridge is fresh");
                prop_assert_eq!(inside.len(), range.length as usize);
                held.extend_from_slice(inside);
            }
        }
        prop_assert_eq!(held, expected);
    }

    /// A rebuild from the same columns gives the same arrays.
    #[test]
    fn two_rebuilds_from_the_same_columns_agree(count in 0u32..80) {
        let arena = populate(count);
        let mut once = bridge();
        let mut twice = bridge();
        once.rebuild(&arena, 3).expect("the rebuild must succeed");
        twice.rebuild(&arena, 3).expect("the rebuild must succeed");
        twice.rebuild(&arena, 1).expect("the rebuild must succeed");
        for ordinal in 0..(WIDTH * HEIGHT) {
            let place = address(ordinal);
            prop_assert_eq!(
                once.on_tile(&arena, place).expect("fresh"),
                twice.on_tile(&arena, place).expect("fresh")
            );
        }
    }

    /// A block range holds exactly the units inside the block rectangle.
    ///
    /// This is what the block-major key buys. A key that only ordered by
    /// tile would still give a self-consistent range, and it would not put
    /// the tiles of one block in one run.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
    #[test]
    fn a_block_range_holds_exactly_the_units_inside_the_block(count in 0u32..90) {
        let arena = populate(count);
        let mut bridge = bridge();
        bridge.rebuild(&arena, 2).expect("the rebuild must succeed");
        let layout = bridge.layout();

        let mut total = 0usize;
        for block in 0..layout.block_count() {
            let column = block % layout.blocks_wide();
            let row = block / layout.blocks_wide();
            let mut expected: Vec<Entity> = arena
                .iter()
                .filter(|soldier| {
                    let place = arena.address(*soldier).expect("a live soldier has an address");
                    (place.q as u32) >> layout.block_bits() == column
                        && (place.r as u32) >> layout.block_bits() == row
                })
                .collect();
            expected.sort_by_key(|soldier| soldier.to_bits());

            let mut held = bridge
                .in_block(&arena, block)
                .expect("the bridge is fresh")
                .to_vec();
            held.sort_by_key(|soldier| soldier.to_bits());
            prop_assert_eq!(held, expected);
            total += bridge.block_range(block).expect("the block exists").length as usize;
        }
        prop_assert_eq!(total, arena.len() as usize);
    }

    /// The bitplane is set for a block exactly when the block holds a unit.
    #[test]
    fn the_bitplane_marks_the_occupied_blocks(count in 0u32..60) {
        let arena = populate(count);
        let mut bridge = bridge();
        bridge.rebuild(&arena, 2).expect("the rebuild must succeed");
        let layout = bridge.layout();

        let mut occupied = vec![false; layout.block_count() as usize];
        for soldier in arena.iter() {
            let tile = arena.tile(soldier).expect("a live soldier has a tile");
            let key = layout.key_of(tile).expect("every tile has a key");
            occupied[layout.block_of_key(key) as usize] = true;
        }
        for block in 0..layout.block_count() {
            prop_assert_eq!(
                bridge.block_is_occupied(block),
                occupied[block as usize],
                "block {} disagrees with the population", block
            );
        }
    }
}
