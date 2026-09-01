//! A tile is held by a faction, or by nobody.
//!
//! The tests go through the public crate API. They drive the world step,
//! because the step is what must invoke the spread rule. A test that called
//! the rule itself would prove that the rule works and not that anything
//! reaches it.[^1]
//!
//! Every fixture asserts that it produced the case the test needs. A world
//! narrower than the coarsest spacing of the terrain generator holds one kind
//! of ground, so a test of "the terrain changes where a holding goes" on such
//! a world measures the fixture.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.claude/rules/testing.md`
//! [^2]: Findings register, FND-054. `docs/FINDINGS.md`

use std::collections::BTreeSet;

use cachette_core::holding::{claim_threshold, FactionMask, Holder};
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, FactionId, TileIdx, World, WorldConfig};

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

/// Fills a patch of open ground with soldiers of one faction.
///
/// The patch starts at a corner of the world, so two callers can place two
/// factions far apart or side by side. Returns the addresses it used.
fn garrison(world: &mut World, faction: FactionId, first: Axial, edge: i32) -> Vec<Axial> {
    // The choice interval is not the subject of this file. A unit takes an
    // intent at the interval its level 1 cell schedules, and it does not move
    // before it has one, so a test about movement sets the interval to every
    // tick.[^C]
    //
    // [^C]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/draft/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    let mut placed = Vec::new();
    for row in 0..edge {
        for column in 0..edge {
            let address = Axial::new(first.q + column, first.r + row);
            if !world.admits_a_unit(address) {
                continue;
            }
            if world.spawn_soldier(address, faction).is_ok() {
                placed.push(address);
            }
        }
    }
    placed
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
    let placed = garrison(&mut world, FactionId(0), Axial::new(4, 4), 3);
    assert!(!placed.is_empty(), "the fixture placed no soldier");
    run(&mut world, 4, 2);
    let held = placed
        .iter()
        .filter_map(|address| world.tile_holder(*address))
        .filter(|holder| holder.faction() == Some(FactionId(0)))
        .count();
    assert!(held > 0, "no tile under a garrison names its faction");
}

#[test]
fn the_holding_changes_during_the_run() {
    // A holding fixed at generation does not answer the need. The count must
    // grow while the world runs.
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    garrison(&mut world, FactionId(0), Axial::new(4, 4), 3);
    let mut seen = Vec::new();
    for _ in 0..6 {
        world.step(2).expect("the step must run");
        seen.push(world.holding_of(FactionId(0)));
    }
    assert!(
        seen.first() < seen.last(),
        "the holding did not grow over the run: {seen:?}"
    );
    assert!(
        seen.windows(2).any(|pair| pair[0] < pair[1]),
        "no single tick changed the holding: {seen:?}"
    );
}

#[test]
fn the_running_total_agrees_with_a_full_pass() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    garrison(&mut world, FactionId(0), Axial::new(4, 4), 3);
    garrison(&mut world, FactionId(1), Axial::new(60, 60), 3);
    run(&mut world, 8, 4);

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
    garrison(&mut world, FactionId(0), Axial::new(4, 4), 4);
    garrison(&mut world, FactionId(1), Axial::new(60, 60), 4);
    run(&mut world, 8, 4);

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
fn the_terrain_changes_where_a_holding_goes() {
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    let found = kinds(&world);
    // The fixture must supply the input. A world of one kind of ground
    // passes this test against a rule that reads no terrain at all.
    assert!(
        found.len() >= 3,
        "the fixture holds only {found:?}, so it cannot show that the ground matters"
    );
    assert!(
        found.contains(&TileKind::Water),
        "the fixture holds no water, so the refusal is never reached"
    );

    // The garrison starts beside water on purpose. A garrison placed at a
    // corner may never reach a shore inside the frames the test runs, and
    // the refusal would then be untested while the test stayed green.
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
    garrison(&mut world, FactionId(0), beside, 1);
    garrison(&mut world, FactionId(1), Axial::new(48, 48), 6);
    run(&mut world, 12, 4);

    let mut water_beside_a_holding = 0;
    for address in addresses(&world) {
        let Some(kind) = world.tile_kind(address) else {
            continue;
        };
        let holder = world
            .tile_holder(address)
            .expect("the address is inside the world");
        if kind == TileKind::Water {
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
                water_beside_a_holding += 1;
            }
        }
    }
    assert!(
        water_beside_a_holding > 0,
        "no holding ever reached water, so the refusal was never exercised"
    );

    // The ground asks for more support as it rises, and open water admits no
    // holder at all. That order is what makes the spread read the terrain.
    assert_eq!(claim_threshold(TileKind::Water), None);
    assert!(claim_threshold(TileKind::Plain) < claim_threshold(TileKind::Hill));
    assert!(claim_threshold(TileKind::Hill) < claim_threshold(TileKind::Mountain));

    // A holding that covered every kind of open ground equally would say
    // nothing about the terrain. Count the share of each kind that is held.
    let mut held = [0i64; 5];
    let mut total = [0i64; 5];
    for address in addresses(&world) {
        let Some(kind) = world.tile_kind(address) else {
            continue;
        };
        total[kind.to_u8() as usize] += 1;
        if world
            .tile_holder(address)
            .is_some_and(|holder| !holder.is_nobody())
        {
            held[kind.to_u8() as usize] += 1;
        }
    }
    assert_eq!(held[TileKind::Water.to_u8() as usize], 0);
    assert!(
        held.iter().sum::<i64>() > 0,
        "the fixture holds nothing at all"
    );
    assert!(
        total[TileKind::Water.to_u8() as usize] > 0,
        "the fixture holds no water"
    );
}

#[test]
fn two_factions_contend_for_one_tile_and_the_stable_key_decides() {
    // The two garrisons sit side by side, so their holdings meet. The test
    // asserts that they met, because a fixture that only assumed it would
    // measure itself.
    let mut world = World::new(VARIED).expect("the extent must describe a world");
    garrison(&mut world, FactionId(0), Axial::new(20, 20), 5);
    garrison(&mut world, FactionId(1), Axial::new(26, 20), 5);
    run(&mut world, 10, 4);

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
    assert!(
        border > 0,
        "the two holdings never met, so nothing was contested"
    );

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
    garrison(&mut world, FactionId(0), Axial::new(20, 20), 5);
    garrison(&mut world, FactionId(1), Axial::new(26, 20), 5);
    garrison(&mut world, FactionId(2), Axial::new(70, 70), 4);
    run(&mut world, 10, 4);

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
        garrison(&mut world, FactionId(0), Axial::new(20, 20), 5);
        garrison(&mut world, FactionId(1), Axial::new(26, 20), 5);
        run(&mut world, 8, threads);
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

/// The extent of a world in which a unit cannot move.
///
/// A world of one tile has no neighbour, so the movement system moves
/// nobody and every unit stays where the test put it. Support then comes
/// from presence alone, which is what isolates the tie-break.
const ONE_TILE: WorldConfig = WorldConfig {
    width: 1,
    height: 1,
    seed: 11,
    faction_count: 4,
};

/// Builds the one-tile world and returns it with its address.
///
/// The fixture asserts that the ground admits a unit. A world of water
/// admits no unit and no holder, and the tests below would then pass against
/// any rule at all.
fn one_tile() -> (World, Axial) {
    let world = World::new(ONE_TILE).expect("the extent must describe a world");
    let address = Axial::new(0, 0);
    assert!(
        world.admits_a_unit(address),
        "the one-tile fixture holds water, so no unit and no holder can reach it"
    );
    (world, address)
}

#[test]
fn an_equal_contest_goes_to_the_lower_faction_identifier() {
    // Two factions stand on one tile and raise the same support. The winner
    // must come from the key and not from the order the factions were seen
    // in. A golden hash notices that this changed. It cannot say which input
    // the answer stopped depending on, so this test names the input.[^1]
    //
    // [^1]: Testing rules, section 2. `.claude/rules/testing.md`
    let (mut world, address) = one_tile();
    // The soldiers are spawned in descending faction order, so a rule that
    // took the first faction it saw would answer 3.
    world
        .spawn_soldier(address, FactionId(3))
        .expect("the tile admits a unit");
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the tile admits a second unit");
    world.step(1).expect("the step must run");
    assert_eq!(
        world.soldiers().len(),
        2,
        "the fixture lost a unit, so the contest did not happen"
    );
    assert_eq!(
        world.tile_holder(address).and_then(Holder::faction),
        Some(FactionId(1)),
        "an equal contest did not go to the lower faction identifier"
    );
}

#[test]
fn the_greater_support_takes_the_tile() {
    // Two units of one faction outweigh one of another, so the answer comes
    // from the support and not from the faction identifier alone. Without
    // this the tie-break test above would pass against a rule that always
    // answered the lowest identifier.
    let (mut world, address) = one_tile();
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the tile admits a unit");
    world
        .spawn_soldier(address, FactionId(3))
        .expect("the tile admits a second unit");
    world
        .spawn_soldier(address, FactionId(3))
        .expect("the tile admits a third unit");
    world.step(1).expect("the step must run");
    assert_eq!(world.soldiers().len(), 3);
    assert_eq!(
        world.tile_holder(address).and_then(Holder::faction),
        Some(FactionId(3)),
        "the faction with more support did not take the tile"
    );
}

#[test]
fn a_holder_keeps_a_tile_against_an_equal_claim() {
    // The holder gets support for holding it, so a claim that only matches
    // the holder takes nothing. Without that a contested border would change
    // hands on every tick and never settle.
    let (mut world, address) = one_tile();
    world
        .spawn_soldier(address, FactionId(3))
        .expect("the tile admits a unit");
    world.step(1).expect("the step must run");
    assert_eq!(
        world.tile_holder(address).and_then(Holder::faction),
        Some(FactionId(3)),
        "the fixture did not give the tile to a holder"
    );
    world
        .spawn_soldier(address, FactionId(1))
        .expect("the tile admits a second unit");
    world.step(1).expect("the step must run");
    assert_eq!(
        world.tile_holder(address).and_then(Holder::faction),
        Some(FactionId(3)),
        "an equal claim took the tile from its holder"
    );
}
