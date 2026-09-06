//! A tile is held by a faction, or by nobody.
//!
//! The cities decide. A tile is held by the faction of the nearest city that
//! reaches it, and a tile no city reaches is held by nobody.[^1] The tests go
//! through the public crate API and drive the world step, because the step is
//! what must invoke the rule. A test that called the rule itself would prove
//! that the rule works and not that anything reaches it.[^2]
//!
//! Every fixture asserts that it produced the case the test needs. A world
//! narrower than the coarsest spacing of the terrain generator holds one kind
//! of ground, so a test about the ground on such a world measures the
//! fixture.[^3]
//!
//! # References
//!
//! [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^2]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^3]: Findings register, FND-054. `docs/FINDINGS.md`

use std::collections::BTreeSet;

use cachette_core::holding::{FactionMask, Holder};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, TileIdx, World, WorldConfig};

/// The extent of a world that holds more than one kind of ground.
///
/// The generator lays its coarsest lattice over the world, and a world
/// narrower than that spacing samples one cell of it and holds one kind of
/// ground everywhere. Every fixture below therefore asserts the kinds it
/// found rather than assuming them.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-054. `docs/FINDINGS.md`
const VARIED: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// Returns every address of a world, in tile index order.
fn addresses(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect()
}

/// Returns the kinds of ground the world holds.
fn kinds(world: &World) -> BTreeSet<TileKind> {
    addresses(world)
        .into_iter()
        .filter_map(|address| world.tile_kind(address))
        .collect()
}

/// Counts what each faction holds by reading every tile.
///
/// This is the answer that the running total must agree with. It is the
/// expensive answer, and it exists so that the cheap one can be checked.
fn count_by_a_full_pass(world: &World) -> Vec<i64> {
    let ceiling = world.config().faction_count.max(1);
    let mut counts = vec![0i64; ceiling as usize];
    for address in addresses(world) {
        if let Some(faction) = world.tile_holder(address).and_then(Holder::faction) {
            counts[faction.0 as usize] += 1;
        }
    }
    counts
}

/// Founds a city of one faction near an address, and returns its identity.
///
/// A city stands on ground that admits a unit, so the search takes the first
/// such address at or after the one the caller named. The test asserts on the
/// ground the city reaches, and not on the address it was given.
fn city(world: &mut World, faction: FactionId, first: Axial) -> Entity {
    for row in 0..12 {
        for column in 0..12 {
            let address = Axial::new(first.q + column, first.r + row);
            if !world.admits_a_unit(address) {
                continue;
            }
            if world.settlement_on(address).is_some() {
                continue;
            }
            if let Ok(site) = world.found_settlement(address, faction) {
                return site;
            }
        }
    }
    panic!("no ground near {first:?} carries a city");
}

/// Runs the frames and checks the invariants after each one.
fn run(world: &mut World, frames: u64, threads: usize) {
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
        assert!(world.check_invariants(), "the world broke an invariant");
    }
}

#[test]
fn a_new_world_is_held_by_nobody() {
    let world = World::new(VARIED).expect("the extent must describe a world");
    let holder = world
        .tile_holder(Axial::new(0, 0))
        .expect("the address is inside the world");
    assert!(holder.is_nobody(), "a new world must be held by nobody");
    assert_eq!(world.holding().held_tiles(), 0);
    assert!(world.tile_holder(Axial::new(-1, 0)).is_none());
}

#[test]
fn a_tile_answers_who_holds_it() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    let site = city(&mut world, FactionId(0), Axial::new(4, 4));
    let seat = world
        .settlements()
        .address(site)
        .expect("the city is a live settlement");
    run(&mut world, 1, 2);
    assert_eq!(
        world.tile_holder(seat).and_then(Holder::faction),
        Some(FactionId(0)),
        "the tile under a city does not name its faction"
    );
    assert_eq!(world.holds(FactionId(0), seat), Some(true));
    assert_eq!(world.holds(FactionId(1), seat), Some(false));
    assert_eq!(world.holds(FactionId(0), Axial::new(-1, 0)), None);
}

#[test]
fn a_faction_holds_ground_from_the_step_after_it_founds_a_city() {
    // A holding fixed at generation does not answer the need. The count must
    // change while the world runs, and it must follow the cities.
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    assert_eq!(world.holding_of(FactionId(0)), 0);
    city(&mut world, FactionId(0), Axial::new(20, 20));
    assert_eq!(
        world.holding_of(FactionId(0)),
        0,
        "the founding wrote the holder column outside the step"
    );
    run(&mut world, 1, 2);
    let after = world.holding_of(FactionId(0));
    assert!(after > 0, "a founded city gave its faction no ground");
    run(&mut world, 3, 2);
    assert_eq!(
        world.holding_of(FactionId(0)),
        after,
        "the ground of a city grew with no upgrade finished"
    );
}

#[test]
fn the_running_total_agrees_with_a_full_pass() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    city(&mut world, FactionId(0), Axial::new(4, 4));
    city(&mut world, FactionId(1), Axial::new(60, 60));
    run(&mut world, 4, 4);

    let counted = count_by_a_full_pass(&world);
    assert!(
        counted.iter().sum::<i64>() > 0,
        "the fixture holds nothing, so the comparison proves nothing"
    );
    for (faction, expected) in counted.iter().enumerate() {
        assert_eq!(
            world.holding_of(FactionId(faction as u16)),
            *expected,
            "the running total of faction {faction} disagrees with a full pass"
        );
    }
    assert_eq!(world.holding().held_tiles(), counted.iter().sum::<i64>());
}

#[test]
fn the_level_one_cell_reports_the_holding_exactly() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    city(&mut world, FactionId(0), Axial::new(4, 4));
    city(&mut world, FactionId(1), Axial::new(60, 60));
    run(&mut world, 4, 4);

    let expected: i64 = count_by_a_full_pass(&world).iter().sum();
    assert!(expected > 0, "the fixture holds nothing");
    assert_eq!(world.pyramid().total().held_tiles(), expected);

    // A cell must equal the tiles it covers, and not only the whole level.
    let layout = world.pyramid().layout();
    let mut by_block = vec![0i64; world.pyramid().len()];
    for address in addresses(&world) {
        if world
            .tile_holder(address)
            .is_some_and(|holder| !holder.is_nobody())
        {
            let tile = world.grid().index_of(address).expect("the tile is inside");
            let key = layout.key_of(tile).expect("the tile has a key");
            by_block[layout.block_of_key(key) as usize] += 1;
        }
    }
    for (block, expected) in by_block.iter().enumerate() {
        let cell = world
            .pyramid()
            .cell(block as u32)
            .expect("the block names a cell");
        assert_eq!(
            cell.held_tiles(),
            *expected,
            "cell {block} disagrees with the tiles it covers"
        );
    }
    assert!(
        by_block.iter().filter(|count| **count > 0).count() > 1,
        "the holding sits in one cell, so the combination is untested"
    );
}

#[test]
fn no_faction_ever_holds_water() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    let found = kinds(&world);
    // The fixture must supply the input. A world with no water never reaches
    // the refusal, and the test would pass against a rule that reads no
    // terrain at all.
    assert!(
        found.contains(&TileKind::Water),
        "the fixture holds only {found:?}, so the refusal is never reached"
    );

    // The city stands beside water on purpose. A city far from a shore
    // reaches none, and the refusal would then be untested while the test
    // stayed green.
    let shore = addresses(&world)
        .into_iter()
        .find(|address| {
            world.tile_kind(*address) == Some(TileKind::Water)
                && world
                    .grid()
                    .neighbours(*address)
                    .into_iter()
                    .flatten()
                    .any(|neighbour| world.admits_a_unit(neighbour))
        })
        .expect("the fixture holds a shore");
    let beside = world
        .grid()
        .neighbours(shore)
        .into_iter()
        .flatten()
        .find(|address| world.admits_a_unit(*address))
        .expect("the shore has open ground beside it");
    city(&mut world, FactionId(0), beside);
    city(&mut world, FactionId(1), Axial::new(48, 48));
    run(&mut world, 2, 4);

    let mut water_inside_a_reach = 0;
    for address in addresses(&world) {
        if world.tile_kind(address) != Some(TileKind::Water) {
            continue;
        }
        let holder = world
            .tile_holder(address)
            .expect("the address is inside the world");
        assert!(holder.is_nobody(), "a faction holds water at {address:?}");
        if world
            .grid()
            .neighbours(address)
            .into_iter()
            .flatten()
            .any(|neighbour| {
                world
                    .tile_holder(neighbour)
                    .is_some_and(|holder| !holder.is_nobody())
            })
        {
            water_inside_a_reach += 1;
        }
    }
    assert!(
        water_inside_a_reach > 0,
        "no city ever reached water, so the refusal was never exercised"
    );
}

#[test]
fn two_cities_of_two_factions_meet_and_no_tile_is_held_twice() {
    // The two cities sit close, so their reaches overlap. The test asserts
    // that they met, because a fixture that only assumed it would measure
    // itself.
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    city(&mut world, FactionId(0), Axial::new(20, 20));
    city(&mut world, FactionId(1), Axial::new(26, 20));
    run(&mut world, 2, 4);

    let mut border = 0;
    for address in addresses(&world) {
        let Some(holder) = world.tile_holder(address).and_then(Holder::faction) else {
            continue;
        };
        let meets_another = world
            .grid()
            .neighbours(address)
            .into_iter()
            .flatten()
            .filter_map(|neighbour| world.tile_holder(neighbour))
            .filter_map(Holder::faction)
            .any(|other| other != holder);
        if meets_another {
            border += 1;
        }
    }
    assert!(border > 0, "the two reaches never met, so no border exists");

    // Exclusivity. Each faction holds a set of tiles, and no tile is in two
    // of them. The sets come from the public query, not from the column.
    let first: BTreeSet<TileIdx> = world.holding().tiles_held_by(FactionId(0)).collect();
    let second: BTreeSet<TileIdx> = world.holding().tiles_held_by(FactionId(1)).collect();
    assert!(!first.is_empty() && !second.is_empty());
    assert!(
        first.is_disjoint(&second),
        "a tile is held by two factions at once"
    );
    assert_eq!(first.len() as i64, world.holding_of(FactionId(0)));
    assert_eq!(second.len() as i64, world.holding_of(FactionId(1)));
}

#[test]
fn a_block_mask_names_every_faction_that_holds_in_it() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    city(&mut world, FactionId(0), Axial::new(20, 20));
    city(&mut world, FactionId(1), Axial::new(24, 20));
    city(&mut world, FactionId(2), Axial::new(70, 70));
    run(&mut world, 2, 4);

    let layout = world.holding().layout();
    let mut expected = vec![FactionMask::EMPTY; world.holding().layout().block_count() as usize];
    for address in addresses(&world) {
        let Some(faction) = world.tile_holder(address).and_then(Holder::faction) else {
            continue;
        };
        let tile = world.grid().index_of(address).expect("the tile is inside");
        let key = layout.key_of(tile).expect("the tile has a key");
        let block = layout.block_of_key(key) as usize;
        expected[block] = expected[block].with(faction);
    }
    for (block, mask) in expected.iter().enumerate() {
        assert_eq!(
            world.holding().block_mask(block as u32),
            Some(*mask),
            "the mask of block {block} disagrees with the tiles it covers"
        );
    }
    assert!(
        expected.iter().any(|mask| mask.len() > 1),
        "no block holds two factions, so the mask is never more than a flag"
    );

    // The blocks a faction holds in come from the masks and not from a pass
    // over the tiles.
    for faction in 0..world.config().faction_count {
        let faction = FactionId(faction);
        let from_masks: Vec<u32> = world.holding().blocks_held_by(faction).collect();
        let from_tiles: Vec<u32> = expected
            .iter()
            .enumerate()
            .filter(|(_, mask)| mask.contains(faction))
            .map(|(block, _)| block as u32)
            .collect();
        assert_eq!(from_masks, from_tiles);
        assert!(!from_masks.is_empty(), "faction {faction:?} holds nothing");
    }

    // The mask a caller reads for a tile is the mask of the block that
    // covers it.
    let address = Axial::new(20, 20);
    let tile = world.grid().index_of(address).expect("the tile is inside");
    let key = layout.key_of(tile).expect("the tile has a key");
    assert_eq!(
        world.holders_near(address),
        world.holding().block_mask(layout.block_of_key(key))
    );
    assert!(world.holders_near(Axial::new(-1, -1)).is_none());
}

#[test]
fn the_holding_is_identical_at_every_thread_count() {
    let mut expected: Option<(u64, Vec<i64>)> = None;
    for threads in [1usize, 2, 12] {
        let mut world = World::new(VARIED).expect("the extent must describe a world");
        city(&mut world, FactionId(0), Axial::new(20, 20));
        city(&mut world, FactionId(1), Axial::new(24, 20));
        city(&mut world, FactionId(2), Axial::new(70, 70));
        run(&mut world, 4, threads);
        let produced = (world.state_hash().finish(), count_by_a_full_pass(&world));
        match &expected {
            None => expected = Some(produced),
            Some(first) => assert_eq!(*first, produced, "the holding differs at {threads} threads"),
        }
    }
    let (_, counts) = expected.expect("the loop ran");
    assert!(
        counts.iter().filter(|count| **count > 0).count() > 1,
        "only one faction held ground, so the contested case was not covered"
    );
}

#[test]
fn a_faction_that_holds_nothing_answers_zero() {
    let world = World::new(VARIED).expect("the extent must describe a world");
    assert_eq!(world.holding_of(FactionId(0)), 0);
    // A faction outside the addressable set answers zero rather than
    // reading past the census.
    assert_eq!(world.holding_of(FactionId(u16::MAX)), 0);
    assert!(world.holding().tiles_held_by(FactionId(0)).next().is_none());
}

#[test]
fn a_holder_names_a_faction_or_nobody() {
    assert!(Holder::NOBODY.is_nobody());
    assert_eq!(Holder::NOBODY.faction(), None);
    assert_eq!(Holder::default(), Holder::NOBODY);
    assert_eq!(Holder::of(FactionId(7)).faction(), Some(FactionId(7)));
    assert!(!Holder::of(FactionId(7)).is_nobody());
    assert_eq!(Holder::of(FactionId(7)).to_bits(), 7);
}

#[test]
fn a_mask_holds_a_set_of_factions() {
    assert!(FactionMask::EMPTY.is_empty());
    assert_eq!(FactionMask::EMPTY.len(), 0);
    let mask = FactionMask::of(FactionId(1)).with(FactionId(5));
    assert!(mask.contains(FactionId(1)) && mask.contains(FactionId(5)));
    assert!(!mask.contains(FactionId(2)));
    assert_eq!(mask.len(), 2);
    // The union is associative and commutative, so a fold gives one answer.
    let other = FactionMask::of(FactionId(2));
    assert_eq!(mask.union(other), other.union(mask));
    assert_eq!(mask.union(other).len(), 3);
    assert_eq!(mask.union(FactionMask::EMPTY), mask);
    // Every faction outside the addressable set takes the reserved bit, so a
    // question about the whole set keeps working.
    let outside = FactionMask::of(FactionId(200));
    assert_eq!(outside, FactionMask::of(FactionId(300)));
    assert_eq!(outside.to_bits(), 1u64 << 63);
}

/// Returns the mask of every block, derived from a full pass over the tiles.
fn masks_by_a_full_pass(world: &World) -> Vec<FactionMask> {
    let layout = world.holding().layout();
    let mut masks = vec![FactionMask::EMPTY; layout.block_count() as usize];
    for address in addresses(world) {
        let Some(faction) = world.tile_holder(address).and_then(Holder::faction) else {
            continue;
        };
        let tile = world.grid().index_of(address).expect("the tile is inside");
        let key = layout.key_of(tile).expect("the tile has a key");
        let block = layout.block_of_key(key) as usize;
        masks[block] = masks[block].with(faction);
    }
    masks
}

/// A block loses a faction bit when that faction's last tile there goes.
///
/// **This is the transition, and the test beside it does not reach one.** That
/// test compares every mask against a full pass after a run, so it proves the
/// masks agree at one moment. A bit that is set and never cleared agrees at
/// every moment in which nothing was vacated.
///
/// The engine keeps a count of the tiles each faction holds in each block, and
/// clears the bit when a count reaches zero.[^1] Nothing rereads a block. A
/// defect that never clears a bit is therefore invisible unless a fixture
/// makes a faction give up the last of its ground somewhere.
///
/// The city that is destroyed is what vacates the blocks, because a faction
/// with no city holds nothing after the next step.[^2]
///
/// # References
///
/// [^1]: Findings register, FND-307. `docs/FINDINGS.md`
/// [^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
#[test]
fn a_block_mask_loses_a_faction_when_its_last_tile_there_goes() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    let first = city(&mut world, FactionId(0), Axial::new(20, 20));
    city(&mut world, FactionId(1), Axial::new(26, 26));
    run(&mut world, 2, 4);

    let before = masks_by_a_full_pass(&world);
    assert!(
        before.iter().any(|mask| mask.contains(FactionId(0))),
        "the first faction holds nothing, so nothing can be vacated"
    );
    assert!(world.destroy_settlement(first));
    run(&mut world, 1, 4);

    let now = masks_by_a_full_pass(&world);
    let mut cleared = 0usize;
    for (block, (was, is)) in before.iter().zip(now.iter()).enumerate() {
        if was.contains(FactionId(0)) && !is.contains(FactionId(0)) {
            cleared += 1;
        }
        assert_eq!(
            world.holding().block_mask(block as u32),
            Some(*is),
            "the mask of block {block} disagrees with its tiles"
        );
    }
    assert!(
        cleared > 0,
        "no block lost a faction, so this test proves nothing about the \
         transition it exists for"
    );
    assert_eq!(world.holding_of(FactionId(0)), 0);
}
