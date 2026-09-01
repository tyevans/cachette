//! Admission grants an intent against the capacity of the target.
//!
//! A move is an intent, and a separate step admits it. Admission sorts the
//! intents by target tile and then by the identity of the unit, scans each
//! target's segment in that order, and admits until the target reaches the
//! capacity of its ground.[^1] The capacity comes from the terrain table, and
//! the movement kernel holds no capacity value of its own.[^2]
//!
//! **A capacity violation is deterministic.** The same intents give the same
//! wrong answer at every thread count, so the thread-count test passes and the
//! state hash matches its golden file. Neither determinism test can see one.
//! Only a test that asserts the invariant can, and that is this suite.[^3]
//!
//! **The invariant is that no tile gains a unit beyond its capacity.** It is
//! not that no tile is ever above its capacity. A spawn does not read the
//! capacity, so a caller may over-fill a tile, and the register holds that
//! open choice.[^4]
//!
//! The tests see only the public crate API.[^5]
//!
//! # References
//!
//! [^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^4]: Decisions register, DEC-020. `docs/DECISIONS.md`
//! [^5]: Testing policy. `docs/TESTING.md`

use std::collections::BTreeMap;

use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig};

/// The thread counts that the admission tests run at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the generator, so
/// the world holds open ground as well as water.
const EXTENT: u32 = 96;

/// The seed of the fixture world.
const SEED: u64 = 0x0cac_4e77_0023;

/// Builds a world of the fixture seed.
fn world_of(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
    })
    .expect("the extent describes a world");
    // The choice interval is not the subject of this file. A unit takes an
    // intent at the interval its level 1 cell schedules, and it does not move
    // before it has one, so a test about movement sets the interval to every
    // tick.[^C]
    //
    // [^C]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world
}

/// Returns every address of a world that admits a unit, in index order.
fn open_tiles(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// Returns how many soldiers stand on each occupied tile.
fn occupancy(world: &World) -> BTreeMap<Axial, u32> {
    let mut counts = BTreeMap::new();
    for soldier in world.soldiers().iter() {
        if let Some(address) = world.soldiers().address(soldier) {
            *counts.entry(address).or_insert(0) += 1;
        }
    }
    counts
}

/// Returns the capacity of the ground at an address.
fn capacity_at(world: &World, address: Axial) -> u32 {
    world.tile_kind(address).map_or(0, TileKind::capacity)
}

/// Crowds one tile and its neighbourhood, so that admission must refuse.
///
/// The fixture is the point of this suite. A population spread thinly over a
/// world never fills a tile, so every intent is admitted and the assertion
/// measures the fixture rather than the rule.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn crowded(seed: u64) -> (World, Vec<Entity>) {
    let mut world = world_of(seed);
    let open = open_tiles(&world);
    // A patch of open ground, taken in index order, so the soldiers sit next
    // to each other and every draw names a tile a neighbour wants.
    let patch: Vec<Axial> = open
        .iter()
        .copied()
        .filter(|address| address.q >= 8 && address.q < 20 && address.r >= 8 && address.r < 20)
        .collect();
    assert!(
        patch.len() >= 16,
        "the seed left only {} open tiles in the patch",
        patch.len()
    );

    let mut kept = Vec::new();
    // Each tile of the patch is filled to the capacity of its own ground. The
    // test reads the capacity rather than naming a number, because the number
    // is content and the movement kernel must not know it either.
    for address in &patch {
        for ordinal in 0..capacity_at(&world, *address) {
            kept.push(
                world
                    .spawn_soldier(*address, FactionId((ordinal % 2) as u16))
                    .expect("the open tile admits a unit"),
            );
        }
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    (world, kept)
}

#[test]
fn the_fixture_crowds_a_tile_past_what_one_frame_can_admit() {
    // Without this, every assertion below could pass on a world where no
    // target was ever full.
    let (world, kept) = crowded(SEED);
    let counts = occupancy(&world);
    let crowded_tiles = counts
        .iter()
        .filter(|(address, count)| **count >= capacity_at(&world, **address))
        .count();
    assert!(
        crowded_tiles > 8,
        "only {crowded_tiles} tiles reached their capacity, so admission is barely exercised"
    );
    assert!(kept.len() > 64, "the fixture holds {} soldiers", kept.len());
}

#[test]
fn no_tile_gains_a_unit_beyond_its_capacity() {
    // This is the assertion that neither determinism test can make. A
    // capacity violation is deterministic, so it passes both of them.
    let (mut world, _) = crowded(SEED);

    for frame in 0..16 {
        let before = occupancy(&world);
        world.step(2).expect("the step must run");
        let after = occupancy(&world);

        for (address, count) in &after {
            let capacity = capacity_at(&world, *address);
            let was = before.get(address).copied().unwrap_or(0);
            // A tile a caller over-filled may stay over its capacity. It may
            // never rise, and it may never rise above the capacity from
            // below.
            assert!(
                *count <= capacity.max(was),
                "the tile {address:?} holds {count} after frame {frame}, \
                 above its capacity {capacity} and above the {was} it held before"
            );
        }
    }
}

#[test]
fn a_rejected_departure_releases_no_room() {
    // The case the record names, built exactly. Three tiles in a line. The
    // middle and the far tile are both full. The unit in the middle intends
    // the far tile and is refused. A rule that counted its intent as a
    // departure would admit a unit from the first tile into the middle, and
    // the middle would end the tick above its capacity.
    //
    // The test drives the engine rather than the admission function, because
    // the engine is what must honour the rule.
    let (mut world, _) = crowded(SEED);
    let counts = occupancy(&world);

    // Every tile that is full and has a full neighbour is an instance of the
    // shape. The fixture must hold one.
    let pairs = counts
        .iter()
        .filter(|(address, count)| **count >= capacity_at(&world, **address))
        .filter(|(address, _)| {
            (0..6).any(|direction| {
                world
                    .grid()
                    .neighbour(**address, direction)
                    .is_some_and(|next| {
                        counts.get(&next).copied().unwrap_or(0) >= capacity_at(&world, next)
                    })
            })
        })
        .count();
    assert!(
        pairs > 0,
        "no full tile in the fixture has a full neighbour, so the rejected \
         departure has no case to fail on"
    );

    for frame in 0..8 {
        let before = occupancy(&world);
        world.step(1).expect("the step must run");
        let after = occupancy(&world);
        for (address, count) in &after {
            let capacity = capacity_at(&world, *address);
            let was = before.get(address).copied().unwrap_or(0);
            assert!(
                *count <= capacity.max(was),
                "the tile {address:?} rose to {count} after frame {frame}, \
                 which is what a departure that never happened buys"
            );
        }
    }
}

#[test]
fn admission_does_not_depend_on_the_thread_count() {
    // The intents are drawn in parallel and admission runs over the joined
    // set. A thread count that changed the joined order would change who is
    // admitted, and this is the test that sees it.
    let expected = run_and_read(SEED, 6, THREAD_COUNTS[0]);
    for threads in &THREAD_COUNTS[1..] {
        assert_eq!(
            run_and_read(SEED, 6, *threads),
            expected,
            "the positions differ at {threads} threads"
        );
    }
}

/// Runs the frames over a crowded world and returns where each soldier stands.
fn run_and_read(seed: u64, frames: u64, threads: usize) -> Vec<Axial> {
    let (mut world, kept) = crowded(seed);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    kept.iter()
        .map(|soldier| {
            world
                .soldiers()
                .address(*soldier)
                .expect("nothing despawned the soldier")
        })
        .collect()
}

#[test]
fn a_refused_unit_holds_the_tile_it_had() {
    // A refusal must leave the unit where it stood. It must not despawn it,
    // must not move it to a default tile, and must not lose it.
    let (mut world, kept) = crowded(SEED);
    let live_before = world.soldiers().len();

    for frame in 0..8 {
        world.step(2).expect("the step must run");
        assert_eq!(
            world.soldiers().len(),
            live_before,
            "the population changed after frame {frame}"
        );
        for soldier in &kept {
            assert!(
                world.soldiers().address(*soldier).is_some(),
                "a soldier vanished after frame {frame}"
            );
        }
    }
}

#[test]
fn a_crowd_still_moves() {
    // A rule that refused everything would satisfy every capacity assertion
    // above and would be inert. Something must get through.
    let (mut world, kept) = crowded(SEED);
    let before: Vec<Axial> = kept
        .iter()
        .map(|soldier| world.soldiers().address(*soldier).expect("alive"))
        .collect();
    for _ in 0..4 {
        world.step(2).expect("the step must run");
    }
    let moved = kept
        .iter()
        .zip(&before)
        .filter(|(soldier, start)| world.soldiers().address(**soldier).as_ref() != Some(*start))
        .count();
    assert!(
        moved > 0,
        "admission refused every one of the {} soldiers over four frames",
        kept.len()
    );
}

#[test]
fn a_sparse_world_admits_every_intent() {
    // Capacity must not refuse a unit that nothing contends with. A rule
    // that refused in the open would pass every test above.
    let mut world = world_of(SEED);
    let open = open_tiles(&world);
    // Every hundredth open tile, so no two soldiers start within reach of
    // one tile.
    let mut kept = Vec::new();
    for address in open.iter().step_by(53) {
        kept.push(
            world
                .spawn_soldier(*address, FactionId(0))
                .expect("the open tile admits a unit"),
        );
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    assert!(kept.len() > 32, "the fixture holds {} soldiers", kept.len());

    let before: Vec<Axial> = kept
        .iter()
        .map(|soldier| world.soldiers().address(*soldier).expect("alive"))
        .collect();
    world.step(2).expect("the step must run");
    let moved = kept
        .iter()
        .zip(&before)
        .filter(|(soldier, start)| world.soldiers().address(**soldier).as_ref() != Some(*start))
        .count();
    // A soldier stays put when its draw names water or a tile outside the
    // world. Most must move, or admission is refusing in the open.
    assert!(
        moved * 2 > kept.len(),
        "only {moved} of {} soldiers moved in an empty world",
        kept.len()
    );
}

#[test]
fn a_world_in_which_nothing_moves_still_answers_after_a_step() {
    // The step rebuilds the derived structure when the arena has moved since
    // the last rebuild, and not otherwise. A frame in which admission granted
    // nothing leaves the arena untouched, so the structure is not rebuilt,
    // and this is the path that a stalled world takes.
    //
    // The structure must still answer afterwards. A skip that left it unable
    // to answer would be a skip that traded a guarantee.
    //
    // A world of one tile is the stalled world. Every neighbour of its only
    // tile lies outside the extent, and the world does not wrap, so no draw
    // ever names a tile. The generator makes coherent ground and no one-tile
    // island, so this is the shape that exists rather than the one the record
    // pictures.
    let mut world = None;
    for seed in 0..64u64 {
        let candidate = World::new(WorldConfig {
            width: 1,
            height: 1,
            seed,
            faction_count: 1,
        })
        .expect("the extent describes a world");
        if candidate.admits_a_unit(Axial::new(0, 0)) {
            world = Some(candidate);
            break;
        }
    }
    let mut world = world.expect("no seed of the first sixty-four gave a world of open ground");
    let only = Axial::new(0, 0);
    let soldier = world
        .spawn_soldier(only, FactionId(0))
        .expect("the tile admits a unit");
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    let revision_before = world.soldiers().revision();
    for frame in 0..8 {
        world.step(1).expect("the step must run");
        assert_eq!(
            world.soldiers().address(soldier),
            Some(only),
            "the soldier left the only tile after frame {frame}"
        );
        assert_eq!(
            world
                .soldier_count_on(only)
                .expect("the structure must answer after a step"),
            1,
            "the structure lost the soldier after frame {frame}"
        );
    }
    assert_eq!(
        world.soldiers().revision(),
        revision_before,
        "the arena moved, so the stalled path was never taken"
    );
}
