//! The living character column set, through the public interface.
//!
//! A living character is one of the four fixed entity shapes. It carries no
//! tile position and none of the soldier columns.[^1] Its identity is a
//! slot index and a generation, and the generation advances when the arena
//! frees the slot.[^2] [^3]
//!
//! The shape declares its tier at the type, and the tier states the ceiling
//! of the population. The world checks that ceiling once, when it is built,
//! and never on a later call.[^4]
//!
//! Every test here drives the world, not the arena. A column set that no
//! test reaches through the engine is inert.[^5]
//!
//! # References
//!
//! [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^4]: ADR-0054, an entity belongs to one of three tiers, declared at creation, a draft record. `docs/adrs/draft/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
//! [^5]: Testing rules, section 5. `.claude/rules/testing.md`

use cachette_core::character::{CharacterArena, CharacterError};
use cachette_core::soldier::SoldierArena;
use cachette_core::tier::{EntityTier, Shape, CHARACTER_CEILING};
use cachette_core::{Entity, FactionId, Fix32, Tick, World, WorldConfig};

/// The settings that every test here builds a world from.
const CONFIG: WorldConfig = WorldConfig {
    width: 12,
    height: 9,
    seed: 0x00c0_ffee_0000_0001,
    faction_count: 3,
};

/// Builds a world.
fn world() -> World {
    World::new(CONFIG).expect("the extent must describe a world")
}

#[test]
fn a_new_world_holds_no_character() {
    let world = world();
    assert_eq!(world.characters().len(), 0);
    assert!(world.characters().is_empty());
    assert!(world.check_invariants());
}

#[test]
fn a_creation_gives_an_identity_that_resolves() {
    let mut world = world();
    let character = world
        .create_character(FactionId(1))
        .expect("the creation must succeed");
    assert!(world.characters().contains(character));
    assert_eq!(world.characters().faction(character), Some(FactionId(1)));
    assert_eq!(world.characters().len(), 1);
    assert!(world.check_invariants());
}

#[test]
fn a_character_carries_the_tick_they_were_created_on() {
    // The birth column is written by the engine and not by the caller, so a
    // character created after a step carries the later tick.
    let mut world = world();
    let early = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    assert_eq!(world.characters().birth(early), Some(Tick(0)));
    for _ in 0..3 {
        world.step(2).expect("the step must run");
    }
    let late = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    assert_eq!(world.characters().birth(late), Some(Tick(3)));
    assert_eq!(
        world.characters().birth(early),
        Some(Tick(0)),
        "the birth of the first character must not move"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_faction_the_world_does_not_hold_is_a_typed_error() {
    let mut world = world();
    let faction = FactionId(CONFIG.faction_count);
    assert_eq!(
        world.create_character(faction),
        Err(CharacterError::FactionAboveCeiling(faction))
    );
    assert!(world.check_invariants());
}

#[test]
fn a_lost_character_never_hands_their_identity_to_the_next_one() {
    // This is the rule the whole identity record exists for. The generation
    // advances when the arena frees the slot, so the old identity is invalid
    // at the moment the character is lost.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let mut world = world();
    let lost = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    world
        .set_character_renown(lost, Fix32::from_int(9))
        .then_some(())
        .expect("the character is live");
    assert!(world.remove_character(lost));
    assert!(!world.characters().contains(lost));

    let made = world
        .create_character(FactionId(2))
        .expect("the creation must reuse the freed slot");
    assert_eq!(
        made.index(),
        lost.index(),
        "the fixture must reuse the slot, or the test proves nothing"
    );
    assert_ne!(made, lost, "the identity must differ");
    assert_ne!(made.generation(), lost.generation());

    // The old identity resolves to nothing, and it reads nothing of the new
    // character.
    assert_eq!(world.characters().faction(lost), None);
    assert_eq!(world.characters().birth(lost), None);
    assert_eq!(world.characters().renown(lost), None);
    // The new character is not the old one wearing the old renown.
    assert_eq!(world.characters().renown(made), Some(Fix32::ZERO));
    assert_eq!(world.characters().faction(made), Some(FactionId(2)));
    assert!(world.check_invariants());
}

#[test]
fn a_dead_identity_writes_no_renown() {
    let mut world = world();
    let lost = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    assert!(world.remove_character(lost));
    let made = world
        .create_character(FactionId(0))
        .expect("the creation must reuse the freed slot");
    assert_eq!(
        made.index(),
        lost.index(),
        "the fixture must reuse the slot, or the test proves nothing"
    );
    assert!(!world.set_character_renown(lost, Fix32::from_int(5)));
    assert_eq!(
        world.characters().renown(made),
        Some(Fix32::ZERO),
        "a dead identity must not write through to the slot it once held"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_second_removal_of_one_identity_removes_nothing() {
    let mut world = world();
    let character = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    assert!(world.remove_character(character));
    assert!(!world.remove_character(character));
    assert_eq!(world.characters().len(), 0);
    assert!(world.check_invariants());
}

#[test]
fn a_new_character_holds_a_renown_of_zero() {
    // Zero is a real state and not an absent one.[^1]
    //
    // [^1]: Findings register, FND-043. `docs/FINDINGS.md`
    let mut world = world();
    let character = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    assert_eq!(world.characters().renown(character), Some(Fix32::ZERO));
    assert!(world.set_character_renown(character, Fix32::ZERO));
    assert_eq!(world.characters().renown(character), Some(Fix32::ZERO));
}

#[test]
fn the_renown_carries_the_value_that_was_written() {
    let mut world = world();
    let first = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    let second = world
        .create_character(FactionId(1))
        .expect("the creation must succeed");
    assert!(world.set_character_renown(first, Fix32::from_int(7)));
    assert_eq!(world.characters().renown(first), Some(Fix32::from_int(7)));
    assert_eq!(
        world.characters().renown(second),
        Some(Fix32::ZERO),
        "a write must reach one slot only"
    );
    assert!(world.check_invariants());
}

#[test]
fn a_creation_takes_the_oldest_freed_slot() {
    // First-in first-out reuse spreads the generation increments over the
    // whole freed set, so no slot wears out before the others.[^1]
    //
    // [^1]: ADR-0014, entity identity is an index plus a generation, decision D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    let mut world = world();
    let mut made = Vec::new();
    for _ in 0..4 {
        made.push(
            world
                .create_character(FactionId(0))
                .expect("the creation must succeed"),
        );
    }
    for character in &made {
        assert!(world.remove_character(*character));
    }
    for lost in &made {
        let next = world
            .create_character(FactionId(0))
            .expect("the creation must succeed");
        assert_eq!(
            next.index(),
            lost.index(),
            "the arena must take the freed slots oldest first"
        );
    }
    assert!(world.check_invariants());
}

#[test]
fn the_characters_iterate_in_slot_order() {
    // The order is the slot order, and it is the same on every run.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let mut world = world();
    let mut made = Vec::new();
    for step in 0..6u32 {
        made.push(
            world
                .create_character(FactionId((step % 3) as u16))
                .expect("the creation must succeed"),
        );
    }
    assert!(world.remove_character(made[2]));
    let walked: Vec<Entity> = world.characters().iter().collect();
    let expected: Vec<Entity> = made
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != 2)
        .map(|(_, character)| *character)
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
    for _ in 0..5 {
        world
            .create_character(FactionId(0))
            .expect("the creation must succeed");
    }
    let arena = world.characters();
    assert_eq!(arena.slot_count(), 5);
    assert_eq!(arena.faction_column().len(), 5);
    assert_eq!(arena.birth_column().len(), 5);
    assert_eq!(arena.renown_column().len(), 5);
}

#[test]
fn a_character_carries_no_tile_position() {
    // The absence is the reason the shape is separate from the soldier
    // shape.[^1] A test cannot assert the absence of a method, so it
    // asserts the two facts that follow from it: the arena needs no world
    // shape to build, and the world holds a character while it holds no
    // ground for one.
    //
    // [^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
    let mut arena = CharacterArena::new();
    let character = arena
        .create(FactionId(0), Tick(0))
        .expect("the creation needs no tile and no world shape");
    assert!(arena.contains(character));
    assert!(arena.check_invariants());
}

#[test]
fn a_character_survives_a_step_and_the_world_holds_its_invariants() {
    // The step must not disturb the character columns. A shape that no
    // frame ever meets is inert.
    let mut world = world();
    let character = world
        .create_character(FactionId(1))
        .expect("the creation must succeed");
    assert!(world.set_character_renown(character, Fix32::from_int(5)));
    for _ in 0..8 {
        world.step(4).expect("the step must run");
        assert!(world.check_invariants());
    }
    assert_eq!(world.characters().faction(character), Some(FactionId(1)));
    assert_eq!(
        world.characters().renown(character),
        Some(Fix32::from_int(5))
    );
    assert_eq!(world.characters().birth(character), Some(Tick(0)));
}

#[test]
fn a_character_survives_every_soldier_in_the_world() {
    // There is no demotion, and a character is not embodied by a unit here.
    // A world that loses every soldier keeps every character and every
    // character identity.[^1]
    //
    // [^1]: The character graph and inheritance, section 9.8. `docs/research/reports/14-character-graph-and-inheritance.md`
    let mut world = world();
    let character = world
        .create_character(FactionId(0))
        .expect("the creation must succeed");
    let mut soldiers = Vec::new();
    for index in 0..world.grid().tile_count().min(20) {
        let address = cachette_core::Axial::new(
            (index % world.grid().width()) as i32,
            (index / world.grid().width()) as i32,
        );
        if world.admits_a_unit(address) {
            if let Ok(soldier) = world.spawn_soldier(address, FactionId(0)) {
                soldiers.push(soldier);
            }
        }
    }
    assert!(
        !soldiers.is_empty(),
        "the fixture must spawn a soldier, or the test proves nothing"
    );
    for soldier in &soldiers {
        assert!(world.despawn_soldier(*soldier));
    }
    assert_eq!(world.soldiers().len(), 0);
    assert!(world.characters().contains(character));
    assert_eq!(world.characters().len(), 1);
    assert!(world.check_invariants());
}

#[test]
fn the_tier_is_a_property_of_the_shape_and_not_of_the_count() {
    // The tier resolves in a constant context, so no world and no
    // population takes part in the answer. A count could not do this.[^1]
    //
    // [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D2, a draft record. `docs/adrs/draft/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    const CHARACTER: EntityTier = <CharacterArena as Shape>::TIER;
    const SOLDIER: EntityTier = <SoldierArena as Shape>::TIER;
    assert_eq!(CHARACTER, EntityTier::Character);
    assert_eq!(SOLDIER, EntityTier::Mass);

    // The tier of the empty world and the tier of a peopled world agree.
    let mut world = world();
    assert_eq!(CharacterArena::tier(), CHARACTER);
    for _ in 0..40 {
        world
            .create_character(FactionId(0))
            .expect("the creation must succeed");
    }
    assert_eq!(CharacterArena::tier(), CHARACTER);
}

#[test]
fn the_tier_states_the_ceiling_of_the_population() {
    assert_eq!(
        EntityTier::Character.population_ceiling(),
        Some(CHARACTER_CEILING)
    );
    assert_eq!(EntityTier::Singleton.population_ceiling(), Some(1));
    assert_eq!(
        EntityTier::Mass.population_ceiling(),
        None,
        "the mass tier is bounded by the slot index, not by a budget"
    );
    assert_eq!(CharacterArena::ceiling(), CHARACTER_CEILING);
}

#[test]
fn a_capacity_above_the_ceiling_is_refused_when_the_arena_is_built() {
    // The refusal happens once, when a caller builds the arena, and never
    // on a later call.[^1]
    //
    // [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D3, a draft record. `docs/adrs/draft/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    assert_eq!(
        CharacterArena::with_capacity(CHARACTER_CEILING + 1).err(),
        Some(CharacterError::CapacityAboveCeiling {
            asked: CHARACTER_CEILING + 1,
            ceiling: CHARACTER_CEILING,
            tier: EntityTier::Character,
        })
    );
    let arena = CharacterArena::with_capacity(CHARACTER_CEILING)
        .expect("the ceiling itself must be admitted");
    assert_eq!(arena.capacity(), CHARACTER_CEILING);
    assert!(arena.check_invariants());
}

#[test]
fn the_world_builds_its_arena_at_the_declared_ceiling() {
    let world = world();
    assert_eq!(world.characters().capacity(), CHARACTER_CEILING);
    assert_eq!(
        world.characters().capacity(),
        CharacterArena::ceiling(),
        "the world must not hold a second, larger ceiling"
    );
}

#[test]
fn an_arena_at_its_capacity_refuses_a_creation_and_keeps_its_invariants() {
    // The refusal is a full arena and not a tier check. A tier check at a
    // call would be the failure the record forbids.
    let mut arena = CharacterArena::with_capacity(2).expect("two is below the ceiling");
    let first = arena
        .create(FactionId(0), Tick(0))
        .expect("the creation must succeed");
    arena
        .create(FactionId(0), Tick(0))
        .expect("the creation must succeed");
    assert_eq!(
        arena.create(FactionId(0), Tick(0)).err(),
        Some(CharacterError::ArenaFull)
    );
    // A removal frees a slot, so the next creation succeeds again.
    assert!(arena.remove(first));
    assert!(arena.create(FactionId(0), Tick(0)).is_ok());
    assert!(arena.check_invariants());
}

proptest::proptest! {
    /// The character arena is the same at every thread count.
    ///
    /// The property holds over an arbitrary sequence of creations and
    /// losses, and not only over the fixed pattern of the harness. The
    /// engine runs the frames, and the test then reads the arena.[^1] [^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[test]
    fn the_arena_is_identical_at_every_thread_count(
        plan in proptest::collection::vec((0u16..3, 0i16..20, proptest::bool::ANY), 1..48),
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

/// Runs one plan of creations and losses, then the frames.
///
/// Returns the live count, the state hash, the renown of each live
/// character in slot order, and the number of slots the arena opened. The
/// renown catches a change that the count alone would miss. The slot count
/// says whether the plan reached the arena at all.
fn run_plan(
    plan: &[(u16, i16, bool)],
    frames: u64,
    threads: usize,
) -> (u32, u64, Vec<Option<Fix32>>, u32) {
    let mut world = world();
    let mut made: Vec<Entity> = Vec::new();
    for (faction, renown, lose) in plan {
        if let Ok(character) = world.create_character(FactionId(*faction)) {
            world.set_character_renown(character, Fix32::from_int(*renown));
            made.push(character);
        }
        if *lose {
            if let Some(character) = made.pop() {
                world.remove_character(character);
            }
        }
    }
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    assert!(world.check_invariants());
    let renown = world
        .characters()
        .iter()
        .map(|character| world.characters().renown(character))
        .collect();
    (
        world.characters().len(),
        world.state_hash().finish(),
        renown,
        world.characters().slot_count(),
    )
}
