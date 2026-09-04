//! Luxury resources, and the variety score over them.
//!
//! The test goes through the public crate API. It reaches into no internal
//! module.[^1]
//!
//! Four families of test live here, and they answer different questions.
//!
//! The first family asks what the variety depends on. A determinism test
//! cannot tell a correct answer from a consistently wrong one, so each thing
//! the score depends on gets its own test: change it, and the score must
//! change.[^2] Two worlds that carry different luxuries must differ, and two
//! that carry the same must agree.
//!
//! The second family states the pyramid property. A level 1 cell equals the
//! exact combination of the tiles it covers.[^3] That property is the reason
//! a luxury set is a mask and not a fraction.
//!
//! The third family drives the engine and inspects what it did. A capability
//! that nothing reaches through the engine ships inert.[^4]
//!
//! The fourth family supplies the extremes. A fixture that models the typical
//! case supplies no extreme, and the assertion then never receives the input
//! that would fail it.[^5] The extremes here are a world with no luxury, a
//! world that carries the whole catalogue, and a tile that carries several.
//!
//! # References
//!
//! [^1]: Testing rules, section 6. `.claude/rules/testing.md`
//! [^2]: Testing rules, section 2. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 4. `.claude/rules/testing.md`
//! [^4]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^5]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::luxury::{
    LuxuryError, LuxuryField, LuxuryId, LuxurySet, VarietyLevel, LUXURY_CEILING,
};
use cachette_core::{Accum, Axial, FactionId, Grid, TileIdx, World, WorldConfig};

use proptest::prelude::*;

/// The extent of the fixture world.
///
/// The coarsest lattice of the ground generator spans sixty-four tiles, so
/// this world holds three lattice cells along each axis. A world below that
/// spacing holds one ground everywhere, and a test over it measures the
/// fixture.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const EXTENT: u32 = 192;

/// The seed that every fixture reads.
const SEED: u64 = 0x0123_4567_89ab_cdef;

/// Builds the world under test.
fn world() -> World {
    World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 4,
        unit_capacity: 4096,
    })
    .expect("the extent must describe a world")
}

/// Builds the grid of the fixture world.
fn grid() -> Grid {
    Grid::new(EXTENT, EXTENT).expect("the extent must describe a world")
}

/// Builds a world that carries the given placements.
fn world_with(placements: &[(TileIdx, LuxuryId)]) -> World {
    let mut world = world();
    world
        .seed_luxuries(placements)
        .expect("the placements must describe a field");
    world
}

// -------------------------------------------------------------------------
// Family one. What the score depends on.
// -------------------------------------------------------------------------

/// Two worlds that carry different luxuries report different varieties.
///
/// This is the question the score exists to answer. A score that did not move
/// here would repeat on every run and at every thread count, and both
/// determinism tests would pass over it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[test]
fn two_worlds_of_different_luxuries_differ_in_variety() {
    let one = world_with(&[
        (TileIdx(10), LuxuryId(0)),
        (TileIdx(20), LuxuryId(1)),
        (TileIdx(30), LuxuryId(2)),
    ]);
    let other = world_with(&[(TileIdx(10), LuxuryId(0))]);
    assert_eq!(one.world_variety(), 3);
    assert_eq!(other.world_variety(), 1);
    assert_ne!(one.world_variety(), other.world_variety());
    assert_ne!(one.state_hash(), other.state_hash());
}

/// Two worlds that carry the same luxuries report the same variety.
#[test]
fn two_worlds_of_one_set_of_luxuries_agree() {
    let placements = [
        (TileIdx(10), LuxuryId(0)),
        (TileIdx(20), LuxuryId(1)),
        (TileIdx(30), LuxuryId(2)),
    ];
    let one = world_with(&placements);
    let other = world_with(&placements);
    assert_eq!(one.world_variety(), other.world_variety());
    assert_eq!(one.state_hash(), other.state_hash());
}

/// The variety counts different luxuries, not deposits.
///
/// Two tiles that carry one luxury give a variety of one and two deposits.
/// A count that gave two would be counting deposits, and the two questions
/// have different answers.
#[test]
fn one_luxury_on_two_tiles_is_one_variety_and_two_deposits() {
    let world = world_with(&[(TileIdx(10), LuxuryId(5)), (TileIdx(11), LuxuryId(5))]);
    assert_eq!(world.world_variety(), 1);
    assert_eq!(world.luxuries().deposits(), Accum(2));
}

/// The order that the caller lists the placements in changes nothing.
///
/// The engine sorts by tile, so the field, the score and the whole state hash
/// are the same for two callers who listed the same set in two orders.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[test]
fn the_order_of_the_placements_changes_nothing() {
    let ascending = world_with(&[
        (TileIdx(3), LuxuryId(0)),
        (TileIdx(9), LuxuryId(1)),
        (TileIdx(40), LuxuryId(2)),
    ]);
    let scattered = world_with(&[
        (TileIdx(40), LuxuryId(2)),
        (TileIdx(3), LuxuryId(0)),
        (TileIdx(9), LuxuryId(1)),
    ]);
    assert_eq!(ascending.luxuries().tiles(), scattered.luxuries().tiles());
    assert_eq!(ascending.state_hash(), scattered.state_hash());
}

/// A world that carries no luxury hashes differently from one that carries
/// one.
///
/// A luxury is simulated state, so the whole-world hash must report it.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[test]
fn a_luxury_reaches_the_state_hash() {
    let bare = world();
    let seeded = world_with(&[(TileIdx(7), LuxuryId(0))]);
    assert_ne!(bare.state_hash(), seeded.state_hash());
}

/// A seed that places nothing leaves the world where it was.
///
/// The flag that says the world took a seed never reaches the hash, so an
/// empty seed and no seed give one hash.
#[test]
fn an_empty_seed_leaves_the_state_hash_alone() {
    let bare = world();
    let seeded = world_with(&[]);
    assert_eq!(bare.state_hash(), seeded.state_hash());
    assert!(seeded.luxuries_seeded());
    assert!(!bare.luxuries_seeded());
}

// -------------------------------------------------------------------------
// Family two. The pyramid property.
// -------------------------------------------------------------------------

/// A level 1 cell holds exactly the luxuries of the tiles it covers.
///
/// The check walks every tile of the world, finds the cell that covers it,
/// and folds the tile into an answer of its own. Every cell must then agree
/// with the level that the engine derived.[^1]
///
/// # References
///
/// [^1]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
fn assert_level_one_equals_level_zero(world: &World) {
    let level = world.variety_level();
    let layout = level.layout();
    let mut expected = vec![LuxurySet::EMPTY; level.len()];
    let mut deposits = vec![Accum(0); level.len()];
    for index in 0..layout.grid().tile_count() {
        let tile = TileIdx(index);
        let set = world.luxuries_at(tile);
        let key = layout.key_of(tile).expect("the tile lies in the world");
        let block = layout.block_of_key(key) as usize;
        expected[block] = expected[block].union(set);
        deposits[block] = Accum(deposits[block].0 + i64::from(set.variety()));
    }
    for block in 0..level.len() {
        let cell = level
            .cell(block as u32)
            .expect("the level holds every cell of the layout");
        assert_eq!(
            cell, expected[block],
            "cell {block} does not hold the luxuries of its tiles"
        );
        assert_eq!(
            level.variety(block as u32),
            Some(expected[block].variety()),
            "cell {block} does not report the variety of its tiles"
        );
        assert_eq!(
            level.deposits(block as u32),
            Some(deposits[block]),
            "cell {block} does not report the deposits of its tiles"
        );
    }
}

/// Level 1 equals the exact count over its level 0 tiles.
#[test]
fn a_level_one_cell_equals_the_count_over_its_tiles() {
    let world = world_with(&[
        (TileIdx(0), LuxuryId(0)),
        (TileIdx(1), LuxuryId(0)),
        (TileIdx(1), LuxuryId(1)),
        (TileIdx(EXTENT * EXTENT - 1), LuxuryId(63)),
        (TileIdx(EXTENT * 40 + 40), LuxuryId(7)),
    ]);
    assert_level_one_equals_level_zero(&world);
}

/// The whole world equals the fold of level 1.
///
/// The union is associative and commutative, so folding the cells gives the
/// same answer as folding the tiles.[^1]
///
/// # References
///
/// [^1]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
#[test]
fn the_fold_of_level_one_equals_the_fold_of_level_zero() {
    let world = world_with(&[
        (TileIdx(5), LuxuryId(2)),
        (TileIdx(5000), LuxuryId(3)),
        (TileIdx(20000), LuxuryId(2)),
    ]);
    assert_eq!(world.variety_level().total(), world.luxuries().set());
    assert_eq!(
        world.variety_level().deposit_total(),
        world.luxuries().deposits()
    );
    assert_eq!(
        world.variety_level().total().variety(),
        world.world_variety()
    );
}

// -------------------------------------------------------------------------
// Family three. Through the engine.
// -------------------------------------------------------------------------

/// The engine carries the field across a step, at every thread count.
///
/// The world is stepped at one thread, at two threads and at twelve threads.
/// Every run must give one hash, and every run must still report the same
/// variety.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decisions D4 and D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[test]
fn a_stepped_world_gives_one_answer_at_every_thread_count() {
    let placements = [
        (TileIdx(11), LuxuryId(0)),
        (TileIdx(11), LuxuryId(4)),
        (TileIdx(9000), LuxuryId(63)),
    ];
    let mut answers = Vec::new();
    for threads in [1usize, 2, 12] {
        let mut world = world_with(&placements);
        for _ in 0..4 {
            world.step(threads).expect("the step must run");
        }
        assert!(world.check_invariants());
        answers.push((world.state_hash(), world.world_variety()));
    }
    assert_eq!(answers[0], answers[1]);
    assert_eq!(answers[0], answers[2]);
    assert_eq!(answers[0].1, 3);
}

/// The world refuses a second seed.
///
/// The field is not a fact of a frame. A second seed would make a reader ask
/// which frame it read.
#[test]
fn the_world_takes_one_seed_only() {
    let mut world = world();
    world
        .seed_luxuries(&[(TileIdx(1), LuxuryId(0))])
        .expect("the first seed must land");
    assert_eq!(
        world.seed_luxuries(&[(TileIdx(2), LuxuryId(1))]),
        Err(LuxuryError::AlreadySeeded)
    );
    assert_eq!(world.world_variety(), 1);
}

/// A refused seed changes nothing.
#[test]
fn a_refused_seed_leaves_the_world_alone() {
    let mut world = world();
    let before = world.state_hash();
    assert_eq!(
        world.seed_luxuries(&[(TileIdx(1), LuxuryId(LUXURY_CEILING))]),
        Err(LuxuryError::IdAboveCeiling(LUXURY_CEILING))
    );
    assert_eq!(world.state_hash(), before);
    assert!(!world.luxuries_seeded());
    assert_eq!(
        world.seed_luxuries(&[(TileIdx(EXTENT * EXTENT), LuxuryId(0))]),
        Err(LuxuryError::NoSuchTile(EXTENT * EXTENT))
    );
    assert_eq!(world.state_hash(), before);
}

/// The world holds its invariants with luxuries on it.
#[test]
fn a_seeded_world_holds_its_invariants() {
    let world = world_with(&[
        (TileIdx(0), LuxuryId(0)),
        (TileIdx(100), LuxuryId(1)),
        (TileIdx(100), LuxuryId(2)),
    ]);
    assert!(world.check_invariants());
}

/// The variety of a faction counts the luxuries on the ground it holds.
///
/// A world that nobody holds gives nothing to every faction. That is the
/// state before anybody claims a tile, and it is what the read must say.
#[test]
fn a_faction_that_holds_nothing_has_no_variety() {
    let world = world_with(&[(TileIdx(10), LuxuryId(0)), (TileIdx(11), LuxuryId(1))]);
    assert_eq!(world.faction_variety(FactionId(0)), 0);
    assert_eq!(world.world_variety(), 2);
}

/// A faction that takes ground takes the luxuries on it.
///
/// The test drives the engine. It seeds luxuries over a patch, puts a
/// garrison of one faction on that patch, and runs frames until the faction
/// holds the ground. The read must then report the luxuries of the tiles that
/// the faction holds, and no other.[^1]
///
/// The check derives the answer a second way, from the holder of each luxury
/// tile. A read that agreed with itself would prove nothing.
///
/// # References
///
/// [^1]: Testing rules, section 5. `.claude/rules/testing.md`
#[test]
fn a_faction_takes_the_luxuries_of_the_ground_it_takes() {
    let mut world = world();
    let grid = grid();
    // The patch sits away from the edge, so a garrison has room around it.
    let corner = Axial::new(60, 60);
    let mut patch = Vec::new();
    for row in 0..8 {
        for column in 0..8 {
            let address = Axial::new(corner.q + column, corner.r + row);
            if world.admits_a_unit(address) {
                patch.push(address);
            }
        }
    }
    assert!(
        patch.len() >= 8,
        "the fixture must hold ground that admits a unit"
    );
    // Each tile of the patch carries a different luxury, so the answer counts
    // the tiles the faction took rather than repeating one bit.
    let placements: Vec<(TileIdx, LuxuryId)> = patch
        .iter()
        .enumerate()
        .map(|(at, address)| {
            let tile = grid
                .index_of(*address)
                .expect("the address lies in the world");
            (tile, LuxuryId((at % usize::from(LUXURY_CEILING)) as u8))
        })
        .collect();
    world
        .seed_luxuries(&placements)
        .expect("the placements must describe a field");
    let holder_faction = FactionId(1);
    for address in &patch {
        let _ = world.spawn_soldier(*address, holder_faction);
    }
    for _ in 0..12 {
        world.step(2).expect("the step must run");
    }
    assert!(world.check_invariants());

    let mut expected = LuxurySet::EMPTY;
    for row in world.luxuries().tiles() {
        let address = grid
            .address_of(row.tile)
            .expect("the tile lies in the world");
        if world
            .tile_holder(address)
            .and_then(cachette_core::Holder::faction)
            == Some(holder_faction)
        {
            expected = expected.union(row.set);
        }
    }
    assert!(
        expected.variety() > 0,
        "the garrison must take some ground before the read means anything"
    );
    assert_eq!(world.faction_variety(holder_faction), expected.variety());
    assert!(world.faction_variety(holder_faction) <= world.world_variety());
    assert_eq!(world.faction_variety(FactionId(3)), 0);
}

// -------------------------------------------------------------------------
// Family four. The extremes.
// -------------------------------------------------------------------------

/// A world with no luxury reports nothing, everywhere.
#[test]
fn a_world_with_no_luxury_reports_nothing() {
    let world = world_with(&[]);
    assert_eq!(world.world_variety(), 0);
    assert_eq!(world.luxuries().deposits(), Accum(0));
    assert!(world.luxuries().is_empty());
    assert_eq!(world.luxuries().len(), 0);
    assert_eq!(world.variety_level().total(), LuxurySet::EMPTY);
    assert_level_one_equals_level_zero(&world);
    assert!(world.check_invariants());
}

/// A world that carries the whole catalogue reports the ceiling.
///
/// The catalogue is 64 luxuries, and one tile carries all of them. The
/// variety is then the ceiling, and it cannot rise further.
#[test]
fn a_world_that_carries_the_whole_catalogue_reports_the_ceiling() {
    let placements: Vec<(TileIdx, LuxuryId)> = (0..LUXURY_CEILING)
        .map(|id| (TileIdx(500), LuxuryId(id)))
        .collect();
    let world = world_with(&placements);
    assert_eq!(world.world_variety(), u32::from(LUXURY_CEILING));
    assert_eq!(world.luxuries().len(), 1);
    assert_eq!(
        world.luxuries().deposits(),
        Accum(i64::from(LUXURY_CEILING))
    );
    assert_eq!(world.luxuries_at(TileIdx(500)).to_bits(), u64::MAX);
    assert_level_one_equals_level_zero(&world);
    assert!(world.check_invariants());
}

/// A world that spreads the whole catalogue over many tiles reports the same
/// variety and more deposits.
#[test]
fn the_catalogue_spread_over_tiles_gives_one_variety_and_many_deposits() {
    let placements: Vec<(TileIdx, LuxuryId)> = (0..LUXURY_CEILING)
        .map(|id| (TileIdx(u32::from(id) * 37), LuxuryId(id)))
        .collect();
    let world = world_with(&placements);
    assert_eq!(world.world_variety(), u32::from(LUXURY_CEILING));
    assert_eq!(world.luxuries().len(), usize::from(LUXURY_CEILING));
    assert_eq!(
        world.luxuries().deposits(),
        Accum(i64::from(LUXURY_CEILING))
    );
    assert_level_one_equals_level_zero(&world);
}

/// A tile that carries several luxuries reports each of them once.
#[test]
fn a_tile_that_carries_several_luxuries_reports_each_once() {
    let world = world_with(&[
        (TileIdx(64), LuxuryId(0)),
        (TileIdx(64), LuxuryId(1)),
        (TileIdx(64), LuxuryId(2)),
        (TileIdx(64), LuxuryId(2)),
    ]);
    let set = world.luxuries_at(TileIdx(64));
    assert_eq!(set.variety(), 3);
    assert!(set.contains(LuxuryId(0)));
    assert!(set.contains(LuxuryId(1)));
    assert!(set.contains(LuxuryId(2)));
    assert!(!set.contains(LuxuryId(3)));
    assert_eq!(world.luxuries().len(), 1);
    assert_level_one_equals_level_zero(&world);
}

/// The catalogue names 64 luxuries, and the identifier above it is refused.
///
/// A set of factions folds an unaddressable faction onto an overflow bit. A
/// luxury set must not, because two luxuries on one bit report the variety as
/// one less than it is.
#[test]
fn the_catalogue_refuses_the_identifier_above_its_ceiling() {
    assert_eq!(LUXURY_CEILING, 64);
    assert!(LuxuryId(LUXURY_CEILING - 1).is_addressable());
    assert!(!LuxuryId(LUXURY_CEILING).is_addressable());
    assert!(LuxurySet::of(LuxuryId(LUXURY_CEILING - 1)).is_some());
    assert!(LuxurySet::of(LuxuryId(LUXURY_CEILING)).is_none());
    assert_eq!(
        LuxuryField::seed(grid(), &[(TileIdx(0), LuxuryId(LUXURY_CEILING))]),
        Err(LuxuryError::IdAboveCeiling(LUXURY_CEILING))
    );
    assert_eq!(
        LuxuryField::seed(grid(), &[(TileIdx(0), LuxuryId(u8::MAX))]),
        Err(LuxuryError::IdAboveCeiling(u8::MAX))
    );
}

/// A tile that carries nothing gives the empty set, inside the world and
/// outside it.
#[test]
fn a_tile_that_carries_nothing_gives_the_empty_set() {
    let world = world_with(&[(TileIdx(1), LuxuryId(0))]);
    assert_eq!(world.luxuries_at(TileIdx(2)), LuxurySet::EMPTY);
    assert_eq!(world.luxuries_at(TileIdx(u32::MAX)), LuxurySet::EMPTY);
    assert_eq!(world.luxuries().variety_at(TileIdx(2)), 0);
}

/// Every stored entry declares its padding, and every padding byte is zero.
///
/// An undeclared padding byte puts an uninitialised byte into the state hash,
/// and such a hash differs between two runs of one binary.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[test]
fn every_stored_entry_declares_its_padding() {
    let world = world_with(&[(TileIdx(1), LuxuryId(0)), (TileIdx(2), LuxuryId(1))]);
    for row in world.luxuries().tiles() {
        assert_eq!(row.padding, [0; 4]);
    }
    assert_eq!(core::mem::size_of::<cachette_core::LuxuryTile>(), 16);
    assert_eq!(core::mem::align_of::<cachette_core::LuxuryTile>(), 8);
    assert_eq!(core::mem::size_of::<LuxurySet>(), 8);
    assert_eq!(core::mem::size_of::<LuxuryId>(), 1);
}

// -------------------------------------------------------------------------
// The algebra.
// -------------------------------------------------------------------------

/// Builds a set from a list of identifiers.
fn set_of(ids: &[u8]) -> LuxurySet {
    ids.iter().fold(LuxurySet::EMPTY, |set, id| {
        set.with(LuxuryId(*id % LUXURY_CEILING))
            .expect("the identifier lies under the ceiling")
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// The union is associative, commutative and idempotent.
    ///
    /// A fold over a group of tiles therefore gives one answer whatever the
    /// order and whatever the grouping.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[test]
    fn the_union_is_a_commutative_idempotent_monoid(
        first in prop::collection::vec(any::<u8>(), 0..8),
        second in prop::collection::vec(any::<u8>(), 0..8),
        third in prop::collection::vec(any::<u8>(), 0..8),
    ) {
        let (a, b, c) = (set_of(&first), set_of(&second), set_of(&third));
        prop_assert_eq!(a.union(b), b.union(a));
        prop_assert_eq!(a.union(b).union(c), a.union(b.union(c)));
        prop_assert_eq!(a.union(a), a);
        prop_assert_eq!(a.union(LuxurySet::EMPTY), a);
        prop_assert_eq!(LuxurySet::EMPTY.union(a), a);
    }

    /// The variety of a union never falls below either part, and never rises
    /// above their sum.
    #[test]
    fn the_variety_of_a_union_lies_between_the_parts_and_their_sum(
        first in prop::collection::vec(any::<u8>(), 0..40),
        second in prop::collection::vec(any::<u8>(), 0..40),
    ) {
        let (a, b) = (set_of(&first), set_of(&second));
        let joined = a.union(b).variety();
        prop_assert!(joined >= a.variety());
        prop_assert!(joined >= b.variety());
        prop_assert!(joined <= a.variety() + b.variety());
        prop_assert!(joined <= u32::from(LUXURY_CEILING));
    }

    /// A field answers the same however the caller ordered the placements,
    /// and level 1 always equals level 0 over it.
    #[test]
    fn a_field_answers_the_same_however_the_placements_arrived(
        tiles in prop::collection::vec(0u32..4096, 0..24),
        ids in prop::collection::vec(any::<u8>(), 0..24),
    ) {
        let count = tiles.len().min(ids.len());
        let mut placements: Vec<(TileIdx, LuxuryId)> = (0..count)
            .map(|at| (TileIdx(tiles[at]), LuxuryId(ids[at] % LUXURY_CEILING)))
            .collect();
        let forward = LuxuryField::seed(grid(), &placements)
            .expect("every placement lies in the world");
        placements.reverse();
        let backward = LuxuryField::seed(grid(), &placements)
            .expect("every placement lies in the world");
        prop_assert_eq!(forward.tiles(), backward.tiles());
        prop_assert!(forward.check_invariants(grid().tile_count()));

        // The deposits are the sum of the variety of every tile, and the set
        // is the union of every tile.
        let mut deposits = 0i64;
        let mut union = LuxurySet::EMPTY;
        for row in forward.tiles() {
            deposits += i64::from(row.set.variety());
            union = union.union(row.set);
        }
        prop_assert_eq!(forward.deposits(), Accum(deposits));
        prop_assert_eq!(forward.set(), union);

        // Level 1 holds exactly what the tiles under it hold.
        let layout = world().variety_level().layout();
        let level = VarietyLevel::derive(layout, &forward);
        prop_assert_eq!(level.total(), forward.set());
        prop_assert_eq!(level.deposit_total(), forward.deposits());
        for row in forward.tiles() {
            let key = layout.key_of(row.tile).expect("the tile lies in the world");
            let block = layout.block_of_key(key);
            let cell = level.cell(block).expect("the level holds the cell");
            prop_assert_eq!(cell.union(row.set), cell);
        }
    }
}
