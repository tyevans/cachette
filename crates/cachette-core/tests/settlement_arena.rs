//! The settlement column set, through the public interface.
//!
//! A settlement is one of the four fixed entity shapes. It is fixed to a
//! tile and it holds pooled stores.[^1] Its identity is a slot index and a
//! generation, and the generation advances when the arena frees the
//! slot.[^2] [^3]
//!
//! Every test here drives the world, not the arena. A column set that no
//! test reaches through the engine is inert.[^4]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^4]: Testing rules, section 5. `.claude/rules/testing.md`

use cachette_core::site::{CommodityId, SettlementError, COMMODITY_COUNT};
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The settings that every test here builds a world from.
const CONFIG: WorldConfig = WorldConfig {
    width: 12,
    height: 9,
    seed: 0x005e_771e_u64,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The only commodity in the set.
const GRAIN: CommodityId = CommodityId(0);

/// Builds a world.
fn world() -> World {
    World::new(CONFIG).expect("the extent must describe a world")
}

/// The extent of the world that holds water as well as open ground.
///
/// The coarsest lattice of the terrain generator spans sixty-four tiles. A
/// world narrower than that spacing sits inside one lattice cell, so every
/// tile of it falls on the same side of the water threshold and the world
/// holds one kind of ground.[^1] This extent holds three lattice cells along
/// each axis, so the ground varies.
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const MIXED_EXTENT: u32 = 192;

/// The settings of the world that holds water.
const MIXED: WorldConfig = WorldConfig {
    width: MIXED_EXTENT,
    height: MIXED_EXTENT,
    seed: 0x0cac_4e77_0092,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Builds a world that holds water and open ground, and returns one of each.
///
/// The fixture asserts over the world it built, not over the seed that made
/// it.[^1] A seed is an input. What the ground turned out to be is the
/// outcome, and the outcome is what these tests need.
///
/// The water tile carries a capacity of zero. That is the whole of what makes
/// the ground refuse a holder, so the fixture states it.
///
/// # References
///
/// [^1]: Findings register, FND-061. `docs/FINDINGS.md`
fn world_of_water_and_ground() -> (World, Axial, Axial) {
    let world = World::new(MIXED).expect("the extent must describe a world");
    let grid = world.grid();
    let mut water = None;
    let mut ground = None;
    for index in 0..grid.tile_count() {
        let address = Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32);
        let kind = world.tile_kind(address).expect("the address is inside");
        if kind.capacity() == 0 && water.is_none() {
            water = Some(address);
        }
        if kind.capacity() > 0 && ground.is_none() {
            ground = Some(address);
        }
    }
    let water = water.expect("the fixture world must hold ground of zero capacity");
    let ground = ground.expect("the fixture world must hold ground that carries a unit");
    assert!(
        !world.admits_a_unit(water),
        "the fixture water tile must refuse a unit"
    );
    assert!(
        world.admits_a_unit(ground),
        "the fixture ground tile must admit a unit"
    );
    (world, water, ground)
}

#[test]
fn a_new_world_holds_no_settlement() {
    let world = world();
    assert_eq!(world.settlements().len(), 0);
    assert!(world.settlements().is_empty());
    assert!(world.check_invariants());
}

#[test]
fn a_founding_gives_an_identity_that_resolves() {
    let mut world = world();
    let address = Axial::new(3, 4);
    let settlement = world
        .found_settlement(address, FactionId(2))
        .expect("the founding must succeed");
    assert!(world.settlements().contains(settlement));
    assert_eq!(world.settlements().address(settlement), Some(address));
    assert_eq!(world.settlements().faction(settlement), Some(FactionId(2)));
    assert_eq!(world.settlements().len(), 1);
    assert!(world.check_invariants());
}

#[test]
fn a_settlement_stands_on_the_tile_that_founded_it() {
    let mut world = world();
    let address = Axial::new(5, 2);
    let settlement = world
        .found_settlement(address, FactionId(0))
        .expect("the founding must succeed");
    assert_eq!(world.settlement_on(address), Some(settlement));
    assert_eq!(world.settlement_on(Axial::new(5, 3)), None);
    assert_eq!(world.settlement_on(Axial::new(-1, 0)), None);
}

#[test]
fn two_settlements_cannot_stand_on_one_tile() {
    // A settlement pools the stores of its tile. Two pools on one tile give
    // every later question about that tile two answers.
    let mut world = world();
    let address = Axial::new(1, 1);
    world
        .found_settlement(address, FactionId(0))
        .expect("the founding must succeed");
    assert_eq!(
        world.found_settlement(address, FactionId(1)),
        Err(SettlementError::TileAlreadyHeld(address))
    );
    assert_eq!(world.settlements().len(), 1);
    assert!(world.check_invariants());
}

#[test]
fn a_lost_settlement_frees_its_tile() {
    let mut world = world();
    let address = Axial::new(2, 2);
    let first = world
        .found_settlement(address, FactionId(0))
        .expect("the founding must succeed");
    assert!(world.destroy_settlement(first));
    assert_eq!(world.settlement_on(address), None);
    let second = world
        .found_settlement(address, FactionId(1))
        .expect("the tile is free again");
    assert_eq!(world.settlement_on(address), Some(second));
    assert!(world.check_invariants());
}

#[test]
fn a_settlement_outside_the_world_is_a_typed_error() {
    // The refusal is a value, not a panic.
    let mut world = world();
    let outside = Axial::new(CONFIG.width as i32, 0);
    assert_eq!(
        world.found_settlement(outside, FactionId(0)),
        Err(SettlementError::TileOutsideWorld(outside))
    );
    let negative = Axial::new(-1, -1);
    assert_eq!(
        world.found_settlement(negative, FactionId(0)),
        Err(SettlementError::TileOutsideWorld(negative))
    );
    assert!(world.check_invariants());
}

#[test]
fn a_faction_the_world_does_not_hold_is_a_typed_error() {
    let mut world = world();
    let faction = FactionId(CONFIG.faction_count);
    assert_eq!(
        world.found_settlement(Axial::new(0, 0), faction),
        Err(SettlementError::FactionAboveCeiling(faction))
    );
    assert!(world.check_invariants());
}

#[test]
fn a_lost_settlement_never_hands_its_identity_to_the_next_one() {
    // This is the rule the whole identity record exists for. The generation
    // advances when the arena frees the slot, so the old identity is invalid
    // at the moment the settlement is lost.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let mut world = world();
    let lost = world
        .found_settlement(Axial::new(0, 0), FactionId(0))
        .expect("the founding must succeed");
    assert!(world.destroy_settlement(lost));
    assert!(!world.settlements().contains(lost));

    let founded = world
        .found_settlement(Axial::new(6, 6), FactionId(1))
        .expect("the founding must reuse the freed slot");
    assert_eq!(
        founded.index(),
        lost.index(),
        "the fixture must reuse the slot, or the test proves nothing"
    );
    assert_ne!(founded, lost, "the identity must differ");
    assert_ne!(founded.generation(), lost.generation());

    // The old identity resolves to nothing, and it reads nothing of the new
    // settlement.
    assert!(!world.settlements().contains(lost));
    assert_eq!(world.settlements().address(lost), None);
    assert_eq!(world.settlements().faction(lost), None);
    assert_eq!(world.settlements().store(lost), None);
    assert!(world.check_invariants());
}

#[test]
fn a_dead_identity_writes_no_store() {
    let mut world = world();
    let lost = world
        .found_settlement(Axial::new(0, 0), FactionId(0))
        .expect("the founding must succeed");
    assert!(world.destroy_settlement(lost));
    let founded = world
        .found_settlement(Axial::new(7, 7), FactionId(0))
        .expect("the founding must reuse the freed slot");
    assert_eq!(founded.index(), lost.index());

    assert_eq!(
        world.set_settlement_store(lost, GRAIN, Fix32::from_int(9)),
        Ok(false)
    );
    assert_eq!(
        world
            .settlements()
            .store(founded)
            .and_then(|store| store.quantity(GRAIN)),
        Some(Fix32::ZERO),
        "a dead handle must not write through to the live settlement"
    );
}

#[test]
fn a_second_loss_of_one_identity_removes_nothing() {
    let mut world = world();
    let settlement = world
        .found_settlement(Axial::new(4, 4), FactionId(0))
        .expect("the founding must succeed");
    assert!(world.destroy_settlement(settlement));
    assert!(!world.destroy_settlement(settlement));
    assert_eq!(world.settlements().len(), 0);
    assert!(world.check_invariants());
}

#[test]
fn a_new_settlement_holds_a_store_of_zero() {
    // Zero is a real state. A settlement that holds nothing of a commodity
    // is not a settlement that holds no store.[^1]
    //
    // [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    let mut world = world();
    let settlement = world
        .found_settlement(Axial::new(0, 0), FactionId(0))
        .expect("the founding must succeed");
    let store = world
        .settlements()
        .store(settlement)
        .expect("a live settlement holds a store");
    assert_eq!(store.quantity(GRAIN), Some(Fix32::ZERO));
}

#[test]
fn a_store_carries_the_quantity_that_was_written() {
    let mut world = world();
    let settlement = world
        .found_settlement(Axial::new(8, 3), FactionId(0))
        .expect("the founding must succeed");
    let quantity = Fix32::from_int(41);
    assert_eq!(
        world.set_settlement_store(settlement, GRAIN, quantity),
        Ok(true)
    );
    assert_eq!(
        world
            .settlements()
            .store(settlement)
            .and_then(|store| store.quantity(GRAIN)),
        Some(quantity)
    );

    // A store written back to zero reads as zero, not as an absent store.
    assert_eq!(
        world.set_settlement_store(settlement, GRAIN, Fix32::ZERO),
        Ok(true)
    );
    assert_eq!(
        world
            .settlements()
            .store(settlement)
            .and_then(|store| store.quantity(GRAIN)),
        Some(Fix32::ZERO)
    );
}

#[test]
fn a_commodity_outside_the_set_is_a_typed_error() {
    let mut world = world();
    let settlement = world
        .found_settlement(Axial::new(0, 0), FactionId(0))
        .expect("the founding must succeed");
    let outside = CommodityId(COMMODITY_COUNT as u16);
    assert_eq!(
        world.set_settlement_store(settlement, outside, Fix32::ONE),
        Err(SettlementError::CommodityOutsideSet(outside))
    );
    assert_eq!(
        world
            .settlements()
            .store(settlement)
            .and_then(|store| store.quantity(outside)),
        None
    );
}

#[test]
fn a_founding_takes_the_oldest_freed_slot() {
    // First-in first-out reuse spreads the generation increments over the
    // whole freed set, so no slot wears out before the others.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let mut world = world();
    let mut founded = Vec::new();
    for step in 0..4u32 {
        founded.push(
            world
                .found_settlement(Axial::new(step as i32, 0), FactionId(0))
                .expect("the founding must succeed"),
        );
    }
    for settlement in &founded {
        assert!(world.destroy_settlement(*settlement));
    }
    for (step, lost) in founded.iter().enumerate() {
        let next = world
            .found_settlement(Axial::new(step as i32, 1), FactionId(0))
            .expect("the founding must succeed");
        assert_eq!(
            next.index(),
            lost.index(),
            "the arena must take the freed slots oldest first"
        );
    }
    assert!(world.check_invariants());
}

#[test]
fn the_settlements_iterate_in_slot_order() {
    // The order is the slot order, and it is the same on every run.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let mut world = world();
    let mut founded = Vec::new();
    for step in 0..6u32 {
        founded.push(
            world
                .found_settlement(Axial::new(step as i32, 2), FactionId(0))
                .expect("the founding must succeed"),
        );
    }
    assert!(world.destroy_settlement(founded[2]));
    let walked: Vec<Entity> = world.settlements().iter().collect();
    let expected: Vec<Entity> = founded
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != 2)
        .map(|(_, settlement)| *settlement)
        .collect();
    assert_eq!(walked, expected);
    let indices: Vec<u32> = walked.iter().map(|entity| entity.index()).collect();
    let mut sorted = indices.clone();
    sorted.sort_unstable();
    assert_eq!(indices, sorted, "the walk must rise through the slots");
}

#[test]
fn the_arena_holds_a_column_for_every_slot() {
    let mut world = world();
    for step in 0..5u32 {
        world
            .found_settlement(Axial::new(step as i32, 5), FactionId(0))
            .expect("the founding must succeed");
    }
    let arena = world.settlements();
    assert_eq!(arena.slot_count(), 5);
    assert_eq!(arena.tile_column().len(), 5);
    assert_eq!(arena.faction_column().len(), 5);
    assert_eq!(arena.store_column().len(), 5);
}

#[test]
fn a_settlement_survives_a_step_and_the_world_holds_its_invariants() {
    // The step must not disturb the settlement columns. A shape that no
    // frame ever meets is inert.
    let mut world = world();
    let settlement = world
        .found_settlement(Axial::new(3, 3), FactionId(1))
        .expect("the founding must succeed");
    world
        .set_settlement_store(settlement, GRAIN, Fix32::from_int(5))
        .expect("the commodity is in the set");
    for _ in 0..8 {
        world.step(4).expect("the step must run");
        assert!(world.check_invariants());
    }
    assert_eq!(
        world.settlements().address(settlement),
        Some(Axial::new(3, 3))
    );
    assert_eq!(
        world
            .settlements()
            .store(settlement)
            .and_then(|store| store.quantity(GRAIN)),
        Some(Fix32::from_int(5))
    );
}

proptest::proptest! {
    /// The settlement arena is the same at every thread count.
    ///
    /// The property holds over an arbitrary sequence of foundings and
    /// losses, and not only over the fixed pattern of the harness. The
    /// engine runs the frames, and the test then reads the arena.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[test]
    fn the_arena_is_identical_at_every_thread_count(
        plan in proptest::collection::vec((0u32..108, 0u16..3, proptest::bool::ANY), 1..48),
        frames in 1u64..4,
    ) {
        let counts = [1usize, 2, 12];
        let first = run_plan(&plan, frames, counts[0]);
        proptest::prop_assert!(
            first.3 > 0,
            "the plan opened no slot, so the run proves nothing"
        );
        for threads in &counts[1..] {
            proptest::prop_assert_eq!(run_plan(&plan, frames, *threads), first.clone());
        }
    }
}

/// Runs one plan of foundings and losses, then the frames.
///
/// Returns the live count, the state hash, the tile of each live settlement
/// in slot order, and the number of slots the arena opened. The tiles catch
/// a change that the count alone would miss. The slot count says whether the
/// plan reached the arena at all.
fn run_plan(
    plan: &[(u32, u16, bool)],
    frames: u64,
    threads: usize,
) -> (u32, u64, Vec<Option<Axial>>, u32) {
    let mut world = world();
    let mut founded: Vec<Entity> = Vec::new();
    for (tile, faction, lose) in plan {
        let address = Axial::new((*tile % CONFIG.width) as i32, (*tile / CONFIG.width) as i32);
        // A tile that already holds a settlement refuses the founding. The
        // refusal is part of the behaviour under test, so the plan keeps it.
        if let Ok(settlement) = world.found_settlement(address, FactionId(*faction)) {
            founded.push(settlement);
        }
        if *lose {
            if let Some(settlement) = founded.pop() {
                world.destroy_settlement(settlement);
            }
        }
    }
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    assert!(world.check_invariants());
    let tiles = world
        .settlements()
        .iter()
        .map(|settlement| world.settlements().address(settlement))
        .collect();
    (
        world.settlements().len(),
        world.state_hash().finish(),
        tiles,
        world.settlements().slot_count(),
    )
}

#[test]
fn a_settlement_on_ground_that_carries_no_unit_is_a_typed_error() {
    // Ground that admits no unit admits no holder.[^1] A settlement is a
    // holder of ground, so the ground that carries nobody carries no
    // settlement. The refusal is a value, not a panic.
    //
    // [^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D5. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
    let (mut world, water, ground) = world_of_water_and_ground();
    assert_eq!(
        world.found_settlement(water, FactionId(0)),
        Err(SettlementError::TileAdmitsNobody(water))
    );
    assert_eq!(world.settlements().len(), 0);
    assert_eq!(world.settlement_on(water), None);

    // The same world takes a settlement on the ground that carries a unit,
    // so the refusal is about the ground and not about the world.
    world
        .found_settlement(ground, FactionId(0))
        .expect("open ground must take a settlement");
    assert_eq!(world.settlements().len(), 1);
    assert!(world.check_invariants());
}

#[test]
fn the_refusal_names_the_ground_and_not_the_extent() {
    // An address outside the world reads no ground at all. The arena owns
    // the extent, so that refusal keeps its own name.
    let (mut world, water, _) = world_of_water_and_ground();
    let outside = Axial::new(MIXED_EXTENT as i32, 0);
    assert_eq!(
        world.found_settlement(outside, FactionId(0)),
        Err(SettlementError::TileOutsideWorld(outside))
    );
    assert_eq!(
        world.found_settlement(water, FactionId(0)),
        Err(SettlementError::TileAdmitsNobody(water))
    );
}
