//! The record of descent, through the public interface.
//!
//! A character records the two characters it came from. A watcher walks
//! from a character to its ancestors and to its descendants, and it reads
//! the relation between any two characters as an exact fixed-point
//! value.[^1]
//!
//! Every test here drives the world. A structure that no test reaches
//! through the engine is inert.[^2]
//!
//! The record of descent is append-only, so it holds a character after that
//! character is gone. The slot columns are not. A death frees the slot, the
//! next character overwrites those columns, and the identity of the
//! character who is gone never resolves again.[^3] [^4]
//!
//! # References
//!
//! [^1]: The character graph and inheritance, section 3.6. `docs/research/reports/14-character-graph-and-inheritance.md`
//! [^2]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^3]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^4]: Decisions register, DEC-003. `docs/DECISIONS.md`

use cachette_core::descent::{DescentId, Parents, RELATION_DEPTH};
use cachette_core::{Entity, FactionId, Fix32, Sex, World, WorldConfig};
use proptest::prelude::*;

/// The settings that every test here builds a world from.
const CONFIG: WorldConfig = WorldConfig {
    width: 8,
    height: 8,
    seed: 0x00c0_ffee_0000_0067,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Builds a world.
fn world() -> World {
    World::new(CONFIG).expect("the extent must describe a world")
}

/// Builds a world from one seed.
fn world_with_seed(seed: u64) -> World {
    World::new(WorldConfig { seed, ..CONFIG }).expect("the extent must describe a world")
}

/// Creates a character who founds a line.
fn founder(world: &mut World) -> Entity {
    world
        .create_character(FactionId(0))
        .expect("the creation must succeed")
}

/// Bears a child of two characters.
fn bear(world: &mut World, mother: Entity, father: Entity) -> Entity {
    world
        .bear_character(mother, father)
        .expect("the birth must succeed")
}

/// Returns the value one half in the fixed-point scale.
fn one_half() -> Fix32 {
    Fix32(Fix32::ONE.0 / 2)
}

/// Returns the value one quarter in the fixed-point scale.
fn one_quarter() -> Fix32 {
    Fix32(Fix32::ONE.0 / 4)
}

#[test]
fn a_character_born_in_the_world_records_both_parents() {
    let mut world = world();
    let mother = founder(&mut world);
    let father = founder(&mut world);
    let child = bear(&mut world, mother, father);

    let parents = world
        .character_parents(child)
        .expect("a living child must answer");
    let mother_id = world
        .characters()
        .descent_id(mother)
        .expect("a living mother holds a row");
    let father_id = world
        .characters()
        .descent_id(father)
        .expect("a living father holds a row");
    assert_eq!(parents.mother, Some(mother_id));
    assert_eq!(parents.father, Some(father_id));
    // The watcher asks who the parent is, and reads back the identity that
    // the arena minted for them.
    let record = world.characters().descent();
    assert_eq!(record.born_as(mother_id), Some(mother));
    assert_eq!(record.born_as(father_id), Some(father));
    assert!(world.check_invariants());
}

#[test]
fn a_character_with_no_parents_is_a_state_and_not_an_invented_pair() {
    let mut world = world();
    let mother = founder(&mut world);
    let father = founder(&mut world);
    let child = bear(&mut world, mother, father);
    let grandchild_mother = founder(&mut world);
    let grandchild = bear(&mut world, grandchild_mother, child);
    let alone = founder(&mut world);

    let parents = world
        .character_parents(alone)
        .expect("a living character must answer");
    assert_eq!(parents, Parents::NONE);
    assert!(parents.is_founder());
    assert!(world.character_ancestors(alone).is_empty());

    // A character raised from the ranks receives no invented ancestry, so
    // the relation to every existing character is zero.[^1]
    //
    // [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    for other in [mother, father, child, grandchild_mother, grandchild] {
        assert_eq!(
            world.character_relation(alone, other),
            Fix32::ZERO,
            "a character who founds a line stands at zero to everybody"
        );
        assert_eq!(world.character_relation(other, alone), Fix32::ZERO);
    }
    // The character still stands at one to itself. A line that starts at
    // zero is not a special case in the recursion.
    assert_eq!(world.character_relation(alone, alone), Fix32::ONE);
    assert!(world.check_invariants());
}

#[test]
fn a_watcher_walks_to_the_ancestors_and_to_the_descendants() {
    // Three generations. The line runs mother, child, grandchild, and each
    // generation takes a partner who founds a line.
    let mut world = world();
    let first = founder(&mut world);
    let first_partner = founder(&mut world);
    let second = bear(&mut world, first, first_partner);
    let second_partner = founder(&mut world);
    let third = bear(&mut world, second, second_partner);
    let third_partner = founder(&mut world);
    let fourth = bear(&mut world, third, third_partner);

    let id = |entity: Entity| {
        world
            .characters()
            .descent_id(entity)
            .expect("a living character holds a row")
    };
    let mut expected_ancestors = vec![
        id(first),
        id(first_partner),
        id(second),
        id(second_partner),
        id(third),
        id(third_partner),
    ];
    expected_ancestors.sort_unstable();
    assert_eq!(world.character_ancestors(fourth), expected_ancestors);

    let mut expected_descendants = vec![id(second), id(third), id(fourth)];
    expected_descendants.sort_unstable();
    assert_eq!(world.character_descendants(first), expected_descendants);
    assert_eq!(world.character_descendants(fourth), Vec::new());
    assert_eq!(world.character_ancestors(first), Vec::new());

    // No character is its own ancestor and no character is its own
    // descendant, at any point in the line.
    for entity in [first, first_partner, second, second_partner, third, fourth] {
        assert!(!world.character_ancestors(entity).contains(&id(entity)));
        assert!(!world.character_descendants(entity).contains(&id(entity)));
    }
    assert!(world.check_invariants());
}

#[test]
fn the_relation_of_a_near_line_is_the_exact_fixed_point_value() {
    let mut world = world();
    let mother = founder(&mut world);
    let father = founder(&mut world);
    let child = bear(&mut world, mother, father);
    let sibling = bear(&mut world, mother, father);
    let outsider = founder(&mut world);
    let grandchild = bear(&mut world, child, outsider);

    // A parent and a child give one half.
    assert_eq!(world.character_relation(mother, child), one_half());
    assert_eq!(world.character_relation(child, mother), one_half());
    // Two children of the same two parents give one half.
    assert_eq!(world.character_relation(child, sibling), one_half());
    // A grandparent and a grandchild give one quarter.
    assert_eq!(world.character_relation(mother, grandchild), one_quarter());
    // The two parents share no ancestor, so they stand at zero.
    assert_eq!(world.character_relation(mother, father), Fix32::ZERO);
    // A character stands at one to itself when nothing inbred them.
    assert_eq!(world.character_relation(child, child), Fix32::ONE);
    assert!(world.check_invariants());
}

#[test]
fn an_inbred_character_raises_the_relation_it_holds_to_itself() {
    // Two children of one pair bear a child together. The child is inbred,
    // so its relation to itself is above one. The value stays exact.
    let mut world = world();
    let mother = founder(&mut world);
    let father = founder(&mut world);
    let daughter = bear(&mut world, mother, father);
    let son = bear(&mut world, mother, father);
    let child = bear(&mut world, daughter, son);

    // The inbreeding coefficient is the kinship of the two parents, which is
    // one quarter. Wright's coefficient of the child with itself is one plus
    // that, which is five quarters.
    assert_eq!(
        world.character_relation(child, child),
        Fix32(Fix32::ONE.0 + one_quarter().0)
    );
    assert!(world.check_invariants());
}

#[test]
fn a_line_that_ends_is_reported_as_ended() {
    let mut world = world();
    let founder_one = founder(&mut world);
    let founder_two = founder(&mut world);
    let child = bear(&mut world, founder_one, founder_two);
    let line = world
        .characters()
        .descent_id(founder_one)
        .expect("a living character holds a row");

    assert!(!world.characters().line_ended(line));
    assert!(world.remove_character(founder_one));
    assert!(
        !world.characters().line_ended(line),
        "a line with a living descendant has not ended"
    );
    assert!(world.remove_character(child));
    assert!(
        world.characters().line_ended(line),
        "a line with no living member has ended"
    );
    // The partner founds a line of their own, and that line ended with the
    // same child.
    let other = world
        .characters()
        .descent_id(founder_two)
        .expect("a living character holds a row");
    assert!(!world.characters().line_ended(other));
    assert!(world.remove_character(founder_two));
    assert!(world.characters().line_ended(other));
    assert!(world.check_invariants());
}

#[test]
fn the_record_of_descent_survives_the_death_of_the_character_it_names() {
    let mut world = world();
    let mother = founder(&mut world);
    let father = founder(&mut world);
    let child = bear(&mut world, mother, father);
    let mother_id = world
        .characters()
        .descent_id(mother)
        .expect("a living mother holds a row");

    assert!(world.remove_character(mother));
    assert!(!world.characters().contains(mother));
    // The slot columns are gone with the character.
    assert_eq!(world.characters().faction(mother), None);
    assert_eq!(world.characters().renown(mother), None);
    assert_eq!(world.characters().sex(mother), None);
    assert_eq!(world.characters().descent_id(mother), None);

    // The descent is not. A watcher reads the dead parent through the
    // living child.
    let parents = world
        .character_parents(child)
        .expect("the child is alive and answers");
    assert_eq!(parents.mother, Some(mother_id));
    let record = world.characters().descent();
    assert_eq!(
        record.born_as(mother_id),
        Some(mother),
        "the record holds the identity the arena minted"
    );
    assert!(!world.characters().is_alive(mother_id));
    assert_eq!(record.children(mother_id).len(), 1);
    assert!(world.character_ancestors(child).contains(&mother_id));
    assert!(world.check_invariants());
}

#[test]
fn a_character_in_a_reused_slot_reaches_none_of_the_ancestors_of_the_dead() {
    // The case this test exists for: a character dies, the arena reuses its
    // slot at a later generation, and a walk from the new character must
    // not reach the ancestors of the character who held the slot before.
    let mut world = world();
    let grandmother = founder(&mut world);
    let grandfather = founder(&mut world);
    let mother = bear(&mut world, grandmother, grandfather);
    let father = founder(&mut world);
    let child = bear(&mut world, mother, father);

    let child_id = world
        .characters()
        .descent_id(child)
        .expect("a living character holds a row");
    let ancestors_of_the_dead = world.character_ancestors(child);
    assert_eq!(ancestors_of_the_dead.len(), 4);

    assert!(world.remove_character(child));
    let successor = founder(&mut world);

    // Prove that the arena reused the slot before asserting anything about
    // the walk. Without the reuse this test proves nothing.
    assert_eq!(
        successor.index(),
        child.index(),
        "the arena must reuse the slot for this test to mean anything"
    );
    assert_eq!(
        successor.generation(),
        child.generation() + 1,
        "the generation must advance on the reuse"
    );

    // The successor holds a row of its own, and it founds a line.
    let successor_id = world
        .characters()
        .descent_id(successor)
        .expect("a living character holds a row");
    assert_ne!(successor_id, child_id);
    assert_eq!(world.character_parents(successor), Some(Parents::NONE));
    assert_eq!(
        world.character_ancestors(successor),
        Vec::new(),
        "the successor must reach none of the ancestors of the dead"
    );
    for ancestor in &ancestors_of_the_dead {
        assert!(!world.character_ancestors(successor).contains(ancestor));
        assert_eq!(
            world.character_relation(
                successor,
                world
                    .characters()
                    .descent()
                    .born_as(*ancestor)
                    .expect("the record holds the identity")
            ),
            Fix32::ZERO,
            "the successor stands at zero to every ancestor of the dead"
        );
    }
    // The identity of the character who is gone reads nothing, and it never
    // reads the successor.
    assert_eq!(world.characters().descent_id(child), None);
    assert_eq!(world.character_parents(child), None);
    assert_eq!(world.character_ancestors(child), Vec::new());
    // The record still holds the dead character and its line.
    assert_eq!(
        world.characters().descent().ancestors(child_id).len(),
        ancestors_of_the_dead.len()
    );
    assert!(world.check_invariants());
}

#[test]
fn the_birth_draw_reads_the_frame() {
    // The same two parents bear a child on twelve different ticks. The draw
    // keys on the tick, so the sex is not the same on every one of them.
    let sexes: Vec<Sex> = (0..12u64)
        .map(|tick| {
            let mut world = world();
            let mother = founder(&mut world);
            let father = founder(&mut world);
            for _ in 0..tick {
                world.step(1).expect("the step must run");
            }
            let child = bear(&mut world, mother, father);
            world
                .characters()
                .sex(child)
                .expect("a living child holds a sex")
        })
        .collect();
    assert!(
        sexes.iter().any(|sex| *sex != sexes[0]),
        "the frame must reach the birth draw, and the draws were {sexes:?}"
    );
}

#[test]
fn the_birth_draw_reads_the_mother() {
    // Twelve mothers bear a child of one father, on one tick, each as the
    // first birth of that mother. Only the mother differs, so the sex is not
    // the same for every one of them.
    let mut world = world();
    let father = founder(&mut world);
    let sexes: Vec<Sex> = (0..12)
        .map(|_| {
            let mother = founder(&mut world);
            let child = bear(&mut world, mother, father);
            world
                .characters()
                .sex(child)
                .expect("a living child holds a sex")
        })
        .collect();
    assert!(
        sexes.iter().any(|sex| *sex != sexes[0]),
        "the mother must reach the birth draw, and the draws were {sexes:?}"
    );
}

#[test]
fn the_birth_draw_reads_the_sequence_of_the_birth() {
    // One mother and one father bear twelve children on one tick. Only the
    // sequence of the birth differs, so the sex is not the same for every
    // one of them.
    let mut world = world();
    let mother = founder(&mut world);
    let father = founder(&mut world);
    let sexes: Vec<Sex> = (0..12)
        .map(|_| {
            let child = bear(&mut world, mother, father);
            world
                .characters()
                .sex(child)
                .expect("a living child holds a sex")
        })
        .collect();
    assert!(
        sexes.iter().any(|sex| *sex != sexes[0]),
        "the sequence must reach the birth draw, and the draws were {sexes:?}"
    );
}

#[test]
fn the_birth_draw_gives_one_answer_for_one_key() {
    // The same seed, the same tick, the same mother and the same sequence
    // give the same draw. The draw holds no state.
    let first = born_sexes(CONFIG.seed);
    let second = born_sexes(CONFIG.seed);
    assert_eq!(first, second);
    let other = born_sexes(CONFIG.seed ^ 0x5555_5555_5555_5555);
    assert_ne!(
        first, other,
        "the seed must reach the birth draw, and the draws were {first:?}"
    );
}

/// Bears twelve children of one pair and returns their sexes.
fn born_sexes(seed: u64) -> Vec<Sex> {
    let mut world = world_with_seed(seed);
    let mother = founder(&mut world);
    let father = founder(&mut world);
    (0..12)
        .map(|_| {
            let child = bear(&mut world, mother, father);
            world
                .characters()
                .sex(child)
                .expect("a living child holds a sex")
        })
        .collect()
}

/// Builds a pedigree from a plan and returns the world.
///
/// Each entry names two earlier characters by position and carries two
/// flags. The first flag steps the world, so the tick moves. The second
/// removes the named character, so the arena frees a slot and the next
/// creation reuses it. **A fixture that never kills anybody never reaches a
/// reused slot**, and a reused slot is where the descent record and the
/// slot columns can disagree.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn build_pedigree(plan: &[(u8, u8, u8)], threads: usize) -> World {
    let mut world = world();
    let mut people = vec![founder(&mut world), founder(&mut world)];
    for (left, right, flags) in plan {
        if flags & 1 == 1 {
            world.step(threads).expect("the step must run");
        }
        if flags & 2 == 2 {
            // The removal is by position, so the plan decides it and the
            // plan is the same at every thread count.
            let victim = people[usize::from(*left) % people.len()];
            world.remove_character(victim);
        }
        let mother = people[usize::from(*left) % people.len()];
        let father = people[usize::from(*right) % people.len()];
        let born = if mother == father || !world.characters().contains(mother) {
            founder(&mut world)
        } else {
            world
                .bear_character(mother, father)
                .unwrap_or_else(|_| founder(&mut world))
        };
        people.push(born);
    }
    world
}

/// Counts the slots that the arena has reused.
///
/// A plan that reuses no slot proves nothing about a reused slot, so the
/// property tests assert on this count.
fn reused_slots(world: &World) -> u32 {
    world.characters().descent().len() - world.characters().slot_count()
}

#[test]
fn the_pedigree_fixture_reaches_a_reused_slot() {
    // The fixture must supply the input that the assertions need. A plan
    // that never kills anybody never frees a slot, so the record of descent
    // and the slot columns are never asked to disagree.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let plan: Vec<(u8, u8, u8)> = (0..12u8).map(|step| (step, step + 1, 3)).collect();
    let world = build_pedigree(&plan, 1);
    assert!(
        reused_slots(&world) > 0,
        "the fixture must reuse a slot, and it reused {}",
        reused_slots(&world)
    );
    assert!(world.check_invariants());
}

/// Returns the whole record of descent as one comparable list.
fn parentage(world: &World) -> Vec<(u32, u64, Option<u32>, Option<u32>)> {
    let record = world.characters().descent();
    (0..record.len())
        .map(|row| {
            let id = record.id_at(row).expect("the row is inside the record");
            let parents = record.parents(id).expect("the row is inside the record");
            (
                row,
                record
                    .born_as(id)
                    .expect("the record holds the identity")
                    .to_bits(),
                parents.mother.map(DescentId::birth_order),
                parents.father.map(DescentId::birth_order),
            )
        })
        .collect()
}

/// Returns the kinship coefficient of two rows, scaled by two to the power
/// of `SCALE`.
///
/// This is the slow reference. It holds no memo, it uses exact integer
/// arithmetic over a common power-of-two denominator, and it never narrows
/// into the fixed-point type. It exists to check the fixed-point recursion
/// against a second implementation.
const SCALE: u32 = 24;

/// Runs the slow reference recursion.
fn reference_kinship(
    parents: &[(Option<u32>, Option<u32>)],
    left: Option<u32>,
    right: Option<u32>,
    depth: u32,
) -> i128 {
    let (Some(left), Some(right)) = (left, right) else {
        return 0;
    };
    if depth == 0 {
        return 0;
    }
    if left == right {
        let (mother, father) = parents[left as usize];
        let inbreeding = reference_kinship(parents, mother, father, depth - 1);
        return ((1i128 << SCALE) + inbreeding) / 2;
    }
    let younger = left.max(right) as usize;
    let older = Some(left.min(right));
    let (mother, father) = parents[younger];
    let from_mother = reference_kinship(parents, mother, older, depth - 1);
    let from_father = reference_kinship(parents, father, older, depth - 1);
    (from_mother + from_father) / 2
}

proptest! {
    /// The relation the world computes equals the slow reference exactly.
    #[test]
    fn the_relation_equals_the_slow_reference(
        plan in proptest::collection::vec((0u8..12, 0u8..12, 0u8..4), 1..14)
    ) {
        let world = build_pedigree(&plan, 1);
        let record = world.characters().descent();
        let rows = record.len();
        let parents: Vec<(Option<u32>, Option<u32>)> = (0..rows)
            .map(|row| {
                let id = record.id_at(row).expect("the row is inside the record");
                let pair = record.parents(id).expect("the row is inside the record");
                (
                    pair.mother.map(DescentId::birth_order),
                    pair.father.map(DescentId::birth_order),
                )
            })
            .collect();
        for left in 0..rows {
            for right in 0..rows {
                let expected = reference_kinship(
                    &parents,
                    Some(left),
                    Some(right),
                    RELATION_DEPTH,
                );
                // Wright's coefficient is twice the kinship coefficient.
                let expected = (expected * 2) >> (SCALE - 16);
                let found = record.relation(
                    record.id_at(left).expect("the row is inside the record"),
                    record.id_at(right).expect("the row is inside the record"),
                );
                prop_assert_eq!(
                    i128::from(found.0),
                    expected,
                    "the relation of row {} and row {} disagreed with the reference",
                    left,
                    right
                );
            }
        }
    }

    /// The relation is symmetric, and no character is its own ancestor.
    #[test]
    fn the_record_holds_its_properties(
        plan in proptest::collection::vec((0u8..12, 0u8..12, 0u8..4), 1..14)
    ) {
        let world = build_pedigree(&plan, 1);
        prop_assert!(world.check_invariants());
        let record = world.characters().descent();
        for row in 0..record.len() {
            let id = record.id_at(row).expect("the row is inside the record");
            let ancestors = record.ancestors(id);
            prop_assert!(!ancestors.contains(&id));
            // The list is sorted and holds no repeat.
            let mut sorted = ancestors.clone();
            sorted.sort_unstable();
            sorted.dedup();
            prop_assert_eq!(&sorted, &ancestors);
            // An ancestor of a character holds that character as a
            // descendant.
            for ancestor in &ancestors {
                prop_assert!(record.descendants(*ancestor).contains(&id));
            }
            for other in 0..record.len() {
                let other = record.id_at(other).expect("the row is inside the record");
                prop_assert_eq!(record.relation(id, other), record.relation(other, id));
            }
        }
    }

    /// The parentage is identical, and recorded in the same order, at one
    /// thread, at two threads and at twelve threads.
    #[test]
    fn the_parentage_is_the_same_at_every_thread_count(
        plan in proptest::collection::vec((0u8..12, 0u8..12, 0u8..4), 1..14)
    ) {
        let expected = build_pedigree(&plan, 1);
        for threads in [2usize, 12] {
            let found = build_pedigree(&plan, threads);
            prop_assert_eq!(parentage(&expected), parentage(&found));
            prop_assert_eq!(
                expected.state_hash().finish(),
                found.state_hash().finish()
            );
        }
    }
}
