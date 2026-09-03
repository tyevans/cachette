//! A character belongs to a house, and a line answers in two comparisons.
//!
//! The record of descent holds three columns beside the parent edges: the
//! house of each character, and the two Euler interval labels of the father
//! forest. A birth copies the house of the father. A relabel pass walks the
//! father forest once and writes the labels, and after it runs "is this
//! character a patrilineal ancestor of that one" is two integer
//! comparisons.[^1]
//!
//! **The fixture is built for the extremes, not for the typical case.** A
//! world of founders and single children would pass every assertion below
//! without exercising any of them. This fixture holds a patrilineal line
//! three generations deep, a sibling that stays behind when the line splits,
//! a maternal edge that must not answer a patrilineal question, and more than
//! one root house, so that a house is never the whole record.[^2]
//!
//! The tests see only the public crate API.[^3]
//!
//! # References
//!
//! [^1]: The character graph and inheritance, sections 3.1 to 3.3. `docs/research/reports/14-character-graph-and-inheritance.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^3]: Testing policy. `docs/TESTING.md`

use cachette_core::{CharacterArena, Entity, FactionId, StateHash, Tick};

/// The seed that every draw in this suite keys on.
const SEED: u64 = 0x0cac_4e77_0097;

/// The faction that every character in the fixture belongs to.
const FACTION: FactionId = FactionId(1);

/// A world whose father forest has depth, breadth and more than one root.
struct Lineage {
    arena: CharacterArena,
    /// The root of the deep line. Its house is the house it founded.
    root: Entity,
    /// The child of the root that carries the line on.
    heir: Entity,
    /// The child of the root that stays behind when the line splits.
    spare: Entity,
    /// The grandchild of the root, through the heir.
    grandchild: Entity,
    /// The great-grandchild of the root, through the grandchild.
    great_grandchild: Entity,
    /// A founder on the maternal side. It fathers nobody in the deep line.
    outsider: Entity,
}

/// Builds the fixture and labels it.
///
/// Every birth names a mother and a father, and the father decides the house.
/// The deep line runs through the father argument at every step, so the
/// patrilineal questions below have a line of depth three to answer over. The
/// mothers are drawn from the other founders, so a maternal edge exists at
/// every generation and a patrilineal answer that followed it would be wrong.
fn lineage() -> Lineage {
    let mut arena = CharacterArena::new();
    let birth = Tick(0);
    let root = arena.create(SEED, FACTION, birth).expect("a free slot");
    let outsider = arena.create(SEED, FACTION, birth).expect("a free slot");
    let other = arena.create(SEED, FACTION, birth).expect("a free slot");

    let heir = arena
        .bear(SEED, outsider, root, birth)
        .expect("two parents");
    let spare = arena
        .bear(SEED, outsider, root, birth)
        .expect("two parents");
    let grandchild = arena.bear(SEED, other, heir, birth).expect("two parents");
    let great_grandchild = arena
        .bear(SEED, other, grandchild, birth)
        .expect("two parents");

    arena.relabel_lines();
    assert!(arena.check_invariants());
    Lineage {
        arena,
        root,
        heir,
        spare,
        grandchild,
        great_grandchild,
        outsider,
    }
}

#[test]
fn the_fixture_supplies_the_cases_the_assertions_need() {
    let world = lineage();
    // A line of depth three. A fixture of depth one would let a parent-edge
    // read pass every ancestry assertion below.
    assert_eq!(
        world
            .arena
            .is_patrilineal_ancestor(world.root, world.great_grandchild),
        Some(true),
        "the fixture holds no line deep enough to test an ancestor walk"
    );
    // More than one root house. A fixture of one house would let a house
    // reader that always answered the same value pass.
    assert_ne!(
        world.arena.house(world.root),
        world.arena.house(world.outsider),
        "the fixture holds one house, so no house assertion can fail"
    );
    // Breadth at the root, so a subtree is not the whole record.
    assert_ne!(world.heir, world.spare);
}

#[test]
fn a_birth_takes_the_house_of_the_father() {
    let world = lineage();
    let house = world.arena.house(world.root).expect("the root is alive");
    for descendant in [
        world.heir,
        world.spare,
        world.grandchild,
        world.great_grandchild,
    ] {
        assert_eq!(
            world.arena.house(descendant),
            Some(house),
            "a character left the house of its father at birth"
        );
    }
    assert_ne!(
        world.arena.house(world.outsider),
        Some(house),
        "the house followed the mother rather than the father"
    );
}

#[test]
fn a_house_is_named_by_the_character_that_founded_it() {
    let world = lineage();
    let house = world.arena.house(world.great_grandchild).expect("alive");
    let founder = house.founder();
    assert_eq!(
        world.arena.descent().born_as(founder),
        Some(world.root),
        "the house of the deep line names a character that did not found it"
    );
}

#[test]
fn a_patrilineal_question_does_not_follow_a_mother() {
    let world = lineage();
    // The outsider is the mother of the heir and a grandmother of the rest.
    // It is an ancestor, and it is not a patrilineal ancestor.
    assert_eq!(
        world
            .arena
            .is_patrilineal_ancestor(world.outsider, world.heir),
        Some(false),
        "a patrilineal question followed a maternal edge"
    );
    let id = world.arena.descent_id(world.outsider).expect("alive");
    let heir = world.arena.descent_id(world.heir).expect("alive");
    assert!(
        world.arena.descent().ancestors(heir).contains(&id),
        "the fixture supplies no maternal edge, so the assertion above is empty"
    );
}

#[test]
fn ancestry_answers_at_every_depth_and_in_one_direction() {
    let world = lineage();
    let ancestor = |high, low| world.arena.is_patrilineal_ancestor(high, low);
    assert_eq!(ancestor(world.root, world.heir), Some(true));
    assert_eq!(ancestor(world.root, world.grandchild), Some(true));
    assert_eq!(ancestor(world.root, world.great_grandchild), Some(true));
    assert_eq!(ancestor(world.heir, world.great_grandchild), Some(true));
    // The relation runs one way only.
    assert_eq!(ancestor(world.great_grandchild, world.root), Some(false));
    // A character is not its own ancestor.
    assert_eq!(ancestor(world.root, world.root), Some(false));
    // A sibling line is not an ancestor line.
    assert_eq!(ancestor(world.spare, world.grandchild), Some(false));
}

#[test]
fn a_line_is_one_span_and_it_never_holds_the_character_itself() {
    let world = lineage();
    let root = world.arena.descent_id(world.root).expect("alive");
    let line = world
        .arena
        .descent()
        .patrilineal_descendants(root)
        .expect("the root is labelled");
    let expected: Vec<_> = [
        world.heir,
        world.spare,
        world.grandchild,
        world.great_grandchild,
    ]
    .iter()
    .map(|entity| world.arena.descent_id(*entity).expect("alive"))
    .collect();
    let mut expected = expected;
    expected.sort_unstable();
    assert_eq!(line, expected);
    assert!(!line.contains(&root));

    // A character at the end of a line has an empty span, not a missing one.
    let leaf = world
        .arena
        .descent_id(world.great_grandchild)
        .expect("alive");
    assert_eq!(
        world.arena.descent().patrilineal_descendants(leaf),
        Some(Vec::new()),
        "the last character of a line answered no span at all"
    );
}

#[test]
fn a_cadet_split_moves_one_line_and_leaves_the_sibling_behind() {
    let mut world = lineage();
    let before = world.arena.house(world.root).expect("alive");
    let moved = world.arena.found_house(world.heir).expect("labelled");
    // The heir, the grandchild and the great-grandchild.
    assert_eq!(moved, 3);

    let cadet = world.arena.house(world.heir).expect("alive");
    assert_ne!(cadet, before, "the split founded no new house");
    assert_eq!(cadet.founder(), world.arena.descent_id(world.heir).unwrap());
    for member in [world.grandchild, world.great_grandchild] {
        assert_eq!(
            world.arena.house(member),
            Some(cadet),
            "a patrilineal descendant of the cadet stayed in the old house"
        );
    }
    for stayer in [world.root, world.spare] {
        assert_eq!(
            world.arena.house(stayer),
            Some(before),
            "the split moved a character outside the line it split"
        );
    }
    assert!(world.arena.check_invariants());
}

#[test]
fn a_house_still_names_its_members_after_a_split() {
    let mut world = lineage();
    world.arena.found_house(world.heir).expect("labelled");
    let cadet = world.arena.house(world.heir).expect("alive");
    let members = world.arena.descent().house_members(cadet);
    let expected: Vec<_> = {
        let mut rows: Vec<_> = [world.heir, world.grandchild, world.great_grandchild]
            .iter()
            .map(|entity| world.arena.descent_id(*entity).expect("alive"))
            .collect();
        rows.sort_unstable();
        rows
    };
    assert_eq!(members, expected);
}

#[test]
fn the_house_of_a_character_survives_its_death() {
    let mut world = lineage();
    let heir_id = world.arena.descent_id(world.heir).expect("alive");
    let house = world.arena.house(world.heir).expect("alive");
    assert!(world.arena.remove(world.heir));
    assert_eq!(
        world.arena.house(world.heir),
        None,
        "a dead identity resolved to a house"
    );
    assert_eq!(
        world.arena.descent().house_of(heir_id),
        Some(house),
        "the record of descent dropped the house of a dead character"
    );
    assert_eq!(
        world.arena.house(world.great_grandchild),
        Some(house),
        "a living line lost its house when a dead ancestor was released"
    );
    assert!(world.arena.check_invariants());
}

#[test]
fn a_character_born_after_the_relabel_carries_no_label() {
    let mut world = lineage();
    let covered = world.arena.descent().labelled_rows();
    assert_eq!(
        covered,
        world.arena.descent().len(),
        "the relabel left a row of the record uncovered"
    );
    let latecomer = world
        .arena
        .bear(SEED, world.outsider, world.great_grandchild, Tick(1))
        .expect("two parents");
    assert_eq!(
        world.arena.descent().labelled_rows(),
        covered,
        "a birth widened the labelled set without a relabel"
    );
    assert_eq!(
        world.arena.is_patrilineal_ancestor(world.root, latecomer),
        None,
        "an unlabelled character answered from a stale label"
    );
    // The house does not wait for the pass. It is written at the birth.
    assert_eq!(
        world.arena.house(latecomer),
        world.arena.house(world.great_grandchild),
        "a birth after the relabel did not take the house of its father"
    );
    world.arena.relabel_lines();
    assert_eq!(
        world.arena.descent().labelled_rows(),
        covered + 1,
        "the relabel did not widen the labelled set to the new row"
    );
    assert_eq!(
        world.arena.is_patrilineal_ancestor(world.root, latecomer),
        Some(true),
        "the relabel did not reach the new character"
    );
    assert!(world.arena.check_invariants());
}

#[test]
fn the_relabel_gives_one_answer_however_often_it_runs() {
    let mut world = lineage();
    let once: Vec<_> = every_label(&world.arena);
    world.arena.relabel_lines();
    let twice: Vec<_> = every_label(&world.arena);
    assert_eq!(once, twice, "a second relabel gave different labels");

    // A second arena built by the same calls gives the same labels. The pass
    // takes its order from the record and never from how it was reached.
    let other = lineage();
    assert_eq!(once, every_label(&other.arena));
}

#[test]
fn a_cadet_split_moves_the_state_hash() {
    let mut world = lineage();
    let before = world.arena.hash_into(StateHash::new()).finish();
    world.arena.found_house(world.heir).expect("labelled");
    let after = world.arena.hash_into(StateHash::new()).finish();
    assert_ne!(
        before, after,
        "the house column does not reach the state hash, so a split is invisible to it"
    );
}

/// Returns the ancestry answer for every ordered pair in the record.
///
/// The vector is the whole of what the labels say. Two records that agree on
/// it agree on every question the labels can answer.
fn every_label(arena: &CharacterArena) -> Vec<Option<bool>> {
    let rows = arena.descent().len();
    let mut out = Vec::new();
    for high in 0..rows {
        for low in 0..rows {
            let high = arena.descent().id_at(high).expect("a row of the record");
            let low = arena.descent().id_at(low).expect("a row of the record");
            out.push(arena.descent().is_patrilineal_ancestor(high, low));
        }
    }
    out
}
