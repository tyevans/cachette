//! The soldier column set and its generational arena.
//!
//! The soldier is one of the four fixed entity shapes, and it has its own
//! set of columns.[^1] A soldier carries a generational identity, a tile
//! address, and a faction.
//!
//! An identity pairs a slot index with a generation, the arena mints every
//! identity, and a dead identity resolves to nothing.[^2] The generation
//! advances at the free, so a dead identity fails at the moment the soldier
//! dies and not at the next reuse.[^3]
//!
//! The tests see only the public crate API.[^4]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^4]: Testing policy. `docs/TESTING.md`

use cachette_core::types::FACTION_CEILING;
use cachette_core::{Axial, Entity, FactionId, Grid, SoldierArena, SoldierError};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// The extent of the world that the tests place soldiers on.
const WIDTH: u32 = 8;
/// The extent of the world that the tests place soldiers on.
const HEIGHT: u32 = 8;

/// Builds an arena over a small world.
fn arena() -> SoldierArena {
    SoldierArena::new(Grid::new(WIDTH, HEIGHT).expect("a small extent describes a grid"))
}

/// Returns the address of a tile index inside the test world.
fn address(index: u32) -> Axial {
    let inside = index % (WIDTH * HEIGHT);
    Axial::new((inside % WIDTH) as i32, (inside / WIDTH) as i32)
}

#[test]
fn the_first_entity_the_arena_ever_allocates_has_an_identity() {
    // ADR-0014 D6: a generation starts at one. Slot zero at generation zero
    // packs to the value zero, which the identity cannot hold, and slot zero
    // is the first slot the arena ever opens. A test that allocated a second
    // entity first would pass without seeing this.
    let mut arena = arena();
    assert!(arena.is_empty());
    let first = arena
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the first spawn must succeed");
    assert_eq!(first.index(), 0);
    assert_eq!(first.generation(), 1);
    assert!(first.to_bits() != 0);
    assert!(arena.contains(first));
    assert_eq!(arena.len(), 1);
    assert!(arena.check_invariants());
}

#[test]
fn a_soldier_carries_a_tile_and_a_faction() {
    let mut arena = arena();
    let soldier = arena
        .spawn(Axial::new(3, 5), FactionId(9))
        .expect("the spawn must succeed");
    assert_eq!(arena.address(soldier), Some(Axial::new(3, 5)));
    assert_eq!(arena.faction(soldier), Some(FactionId(9)));
    assert_eq!(arena.tile(soldier), arena.grid().index_of(Axial::new(3, 5)));
}

#[test]
fn a_stale_identity_fails_at_the_free_and_not_at_the_reuse() {
    // ADR-0014 D3: the generation advances when the arena frees a slot. A
    // generation that advanced on the allocation would leave the stale
    // identity valid until something else claimed the slot.
    let mut arena = arena();
    let soldier = arena
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the spawn must succeed");
    assert!(arena.despawn(soldier));

    // No other spawn has happened. The slot still holds the old columns.
    assert!(!arena.contains(soldier));
    assert_eq!(arena.tile(soldier), None);
    assert_eq!(arena.faction(soldier), None);
    assert_eq!(arena.slot_of(soldier), None);
    assert_eq!(arena.len(), 0);
}

#[test]
fn a_second_despawn_of_one_identity_removes_nothing() {
    let mut arena = arena();
    let soldier = arena
        .spawn(Axial::new(1, 1), FactionId(0))
        .expect("the spawn must succeed");
    assert!(arena.despawn(soldier));
    assert!(!arena.despawn(soldier));
    assert_eq!(arena.len(), 0);
}

#[test]
fn a_reused_slot_gives_a_different_identity() {
    let mut arena = arena();
    let first = arena
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the spawn must succeed");
    assert!(arena.despawn(first));
    let second = arena
        .spawn(Axial::new(2, 2), FactionId(1))
        .expect("the spawn must succeed");
    assert_eq!(first.index(), second.index());
    assert_ne!(first, second);
    assert!(arena.contains(second));
    assert!(!arena.contains(first));
    assert_eq!(arena.slot_count(), 1);
}

#[test]
fn a_freed_slot_returns_in_first_in_first_out_order() {
    // ADR-0014 D4: the arena takes the oldest freed slot. Last-in first-out
    // reuse hands one slot back at once, so that slot takes every generation
    // increment and reaches the end of its range early.
    let mut arena = arena();
    let handles: Vec<Entity> = (0..4)
        .map(|index| {
            arena
                .spawn(address(index), FactionId(0))
                .expect("the spawn must succeed")
        })
        .collect();
    assert!(arena.despawn(handles[0]));
    assert!(arena.despawn(handles[2]));
    assert!(arena.despawn(handles[1]));

    let order: Vec<u32> = (0..3)
        .map(|index| {
            arena
                .spawn(address(index), FactionId(0))
                .expect("the spawn must succeed")
                .index()
        })
        .collect();
    assert_eq!(
        order,
        vec![0, 2, 1],
        "the oldest freed slot must come first"
    );
    assert_eq!(arena.slot_count(), 4, "no reuse may open a new slot");
}

#[test]
fn the_arena_refuses_a_spawn_when_it_holds_no_free_slot() {
    let grid = Grid::new(WIDTH, HEIGHT).expect("a small extent describes a grid");
    let mut arena = SoldierArena::with_capacity(grid, 2);
    let first = arena
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the spawn must succeed");
    arena
        .spawn(Axial::new(1, 0), FactionId(0))
        .expect("the spawn must succeed");
    assert_eq!(
        arena.spawn(Axial::new(2, 0), FactionId(0)),
        Err(SoldierError::ArenaFull)
    );
    // A free slot lets the next spawn through.
    assert!(arena.despawn(first));
    assert!(arena.spawn(Axial::new(2, 0), FactionId(0)).is_ok());
}

#[test]
fn the_arena_refuses_an_address_outside_the_world() {
    let mut arena = arena();
    for outside in [
        Axial::new(-1, 0),
        Axial::new(0, -1),
        Axial::new(WIDTH as i32, 0),
        Axial::new(0, HEIGHT as i32),
    ] {
        assert_eq!(
            arena.spawn(outside, FactionId(0)),
            Err(SoldierError::TileOutsideWorld(outside))
        );
    }
    assert_eq!(arena.slot_count(), 0, "a refusal must open no slot");

    let soldier = arena
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the spawn must succeed");
    assert_eq!(
        arena.place(soldier, Axial::new(WIDTH as i32, 0)),
        Err(SoldierError::TileOutsideWorld(Axial::new(WIDTH as i32, 0)))
    );
    assert_eq!(arena.address(soldier), Some(Axial::new(0, 0)));
}

#[test]
fn the_arena_refuses_a_faction_at_or_above_the_ceiling() {
    let mut arena = arena();
    assert_eq!(
        arena.spawn(Axial::new(0, 0), FactionId(FACTION_CEILING)),
        Err(SoldierError::FactionAboveCeiling(FactionId(
            FACTION_CEILING
        )))
    );
    // The ceiling is exclusive. The highest valid identifier passes.
    assert!(arena
        .spawn(Axial::new(0, 0), FactionId(FACTION_CEILING - 1))
        .is_ok());
}

#[test]
fn a_stale_identity_moves_no_soldier() {
    let mut arena = arena();
    let first = arena
        .spawn(Axial::new(0, 0), FactionId(0))
        .expect("the spawn must succeed");
    assert!(arena.despawn(first));
    let second = arena
        .spawn(Axial::new(1, 1), FactionId(0))
        .expect("the spawn must succeed");
    assert_eq!(
        arena.place(first, Axial::new(7, 7)),
        Ok(false),
        "a stale identity must move nothing"
    );
    assert_eq!(arena.address(second), Some(Axial::new(1, 1)));
    assert_eq!(arena.place(second, Axial::new(7, 7)), Ok(true));
    assert_eq!(arena.address(second), Some(Axial::new(7, 7)));
}

#[test]
fn the_live_soldiers_come_back_in_slot_order() {
    let mut arena = arena();
    let handles: Vec<Entity> = (0..5)
        .map(|index| {
            arena
                .spawn(address(index), FactionId(0))
                .expect("the spawn must succeed")
        })
        .collect();
    assert!(arena.despawn(handles[1]));
    assert!(arena.despawn(handles[3]));
    let live: Vec<u32> = arena.iter().map(Entity::index).collect();
    assert_eq!(live, vec![0, 2, 4]);
}

/// One step of the model, which the property below drives.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// Add a soldier on the tile of this index, for this faction.
    Spawn(u32, u16),
    /// Remove the soldier that the live set holds at this position.
    Despawn(usize),
}

/// A strategy that produces a sequence of steps.
fn any_steps() -> impl Strategy<Value = Vec<Step>> {
    let step = prop_oneof![
        2 => (0u32..256, 0u16..FACTION_CEILING).prop_map(|(tile, faction)| Step::Spawn(tile, faction)),
        1 => (0usize..64).prop_map(Step::Despawn),
    ];
    proptest::collection::vec(step, 0..64)
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
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/soldier_arena.proptest-regressions"),
        ))),
        ..ProptestConfig::default()
    })]

    /// An identity round-trips through the arena.
    ///
    /// The arena gives back the address and the faction that the spawn put
    /// in. This is the property that makes the identity a name for one
    /// soldier.
    #[test]
    fn an_identity_round_trips_through_the_arena(
        tiles in proptest::collection::vec((0u32..256, 0u16..FACTION_CEILING), 1..40)
    ) {
        let mut arena = arena();
        let mut handles = Vec::new();
        for (tile, faction) in &tiles {
            let place = address(*tile);
            let soldier = arena.spawn(place, FactionId(*faction))
                .expect("a valid address and faction must spawn");
            handles.push((soldier, place, FactionId(*faction)));
        }
        for (soldier, place, faction) in &handles {
            prop_assert!(arena.contains(*soldier));
            prop_assert_eq!(arena.address(*soldier), Some(*place));
            prop_assert_eq!(arena.faction(*soldier), Some(*faction));
        }
        prop_assert_eq!(arena.len() as usize, handles.len());
        prop_assert!(arena.check_invariants());
    }

    /// Spawning and despawning in any order leaves exactly the live set.
    ///
    /// The model is a plain list. It holds no generation, so it cannot
    /// repeat a defect of the arena.
    #[test]
    fn any_order_of_spawn_and_despawn_leaves_the_live_set(steps in any_steps()) {
        let mut arena = arena();
        let mut model: Vec<(Entity, Axial, FactionId)> = Vec::new();
        let mut dead: Vec<Entity> = Vec::new();

        for step in &steps {
            match *step {
                Step::Spawn(tile, faction) => {
                    let place = address(tile);
                    let soldier = arena.spawn(place, FactionId(faction))
                        .expect("a valid address and faction must spawn");
                    model.push((soldier, place, FactionId(faction)));
                }
                Step::Despawn(position) => {
                    if model.is_empty() {
                        continue;
                    }
                    let (soldier, _, _) = model.remove(position % model.len());
                    prop_assert!(arena.despawn(soldier));
                    dead.push(soldier);
                }
            }
            prop_assert!(arena.check_invariants());
        }

        prop_assert_eq!(arena.len() as usize, model.len());
        for (soldier, place, faction) in &model {
            prop_assert!(arena.contains(*soldier));
            prop_assert_eq!(arena.address(*soldier), Some(*place));
            prop_assert_eq!(arena.faction(*soldier), Some(*faction));
        }
        // Every identity the arena freed reads as absent, whether or not
        // another soldier now holds the slot.[^1]
        //
        // [^1]: ADR-0014, entity identity is an index plus a generation,
        // decision D2.
        for soldier in &dead {
            prop_assert!(!arena.contains(*soldier));
            prop_assert_eq!(arena.tile(*soldier), None);
        }
        let live: Vec<Entity> = arena.iter().collect();
        let mut expected: Vec<Entity> = model.iter().map(|(soldier, _, _)| *soldier).collect();
        expected.sort_by_key(|soldier| soldier.index());
        prop_assert_eq!(live, expected);
    }

    /// A stale identity reads as absent, never as another soldier.
    #[test]
    fn a_stale_identity_never_reads_as_another_soldier(count in 1usize..24) {
        let mut arena = arena();
        let first: Vec<Entity> = (0..count)
            .map(|index| arena.spawn(address(index as u32), FactionId(0))
                .expect("the spawn must succeed"))
            .collect();
        for soldier in &first {
            prop_assert!(arena.despawn(*soldier));
        }
        // Refill every slot. Each stale identity now names a slot that a
        // live soldier holds.
        let second: Vec<Entity> = (0..count)
            .map(|index| arena.spawn(address(index as u32), FactionId(1))
                .expect("the spawn must succeed"))
            .collect();
        for soldier in &first {
            prop_assert!(!arena.contains(*soldier));
            prop_assert_eq!(arena.faction(*soldier), None);
        }
        for soldier in &second {
            prop_assert_eq!(arena.faction(*soldier), Some(FactionId(1)));
        }
        prop_assert_eq!(arena.slot_count() as usize, count);
    }
}
