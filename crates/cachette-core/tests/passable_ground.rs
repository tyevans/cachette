//! The ground refuses a unit, and every path into a tile honours the refusal.
//!
//! The terrain says whether a unit may stand on a tile.[^1] Until this suite
//! existed, nothing read that answer: a soldier walked into water and no test
//! failed. That is the inert-capability shape, and the rule says the test must
//! start at the engine.[^2]
//!
//! Three paths put a soldier on a tile: a spawn, a placement, and the
//! movement system at the frame barrier. Each one is covered here, and the
//! movement case drives a stepping world rather than the system alone.
//!
//! A refusal by the ground is not a refusal by admission. The ground refuses
//! every unit on every frame, whatever else stands there. Admission is a
//! contest between units for one tile, and it is not built yet.[^3]
//!
//! Every fixture states how much water it put next to a soldier. A fixture
//! that supplies no water supplies no case, and the assertion then measures
//! the fixture.[^4]
//!
//! The tests see only the public crate API.[^5]
//!
//! # References
//!
//! [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4, a draft record. `docs/adrs/draft/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^2]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
//! [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^5]: Testing policy. `docs/TESTING.md`

use cachette_core::{Axial, FactionId, SoldierError, TileKind, World, WorldConfig};

/// The extent of the fixture world.
///
/// The extent is wider than the coarsest lattice spacing of the generator, so
/// the world holds both water and open ground rather than one or the other.
const EXTENT: u32 = 96;

/// The seed of the fixture world.
const SEED: u64 = 0x0cac_4e77_0068;

/// The number of frames that the stepping test runs.
const FRAMES: u64 = 24;

/// Builds the fixture world and returns it with its open and its flooded
/// tiles, each in index order.
fn fixture() -> (World, Vec<Axial>, Vec<Axial>) {
    let world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
    })
    .expect("the extent describes a world");
    let grid = world.grid();
    let every: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .collect();
    let open: Vec<Axial> = every
        .iter()
        .copied()
        .filter(|address| world.admits_a_unit(*address))
        .collect();
    let flooded: Vec<Axial> = every
        .into_iter()
        .filter(|address| !world.admits_a_unit(*address))
        .collect();
    (world, open, flooded)
}

#[test]
fn the_fixture_world_holds_both_water_and_open_ground() {
    // A fixture of all open ground never offers a soldier a tile to be
    // refused, and every assertion below then passes without a case. A
    // fixture of all water holds no soldier at all.
    let (_, open, flooded) = fixture();
    assert!(
        flooded.len() > open.len() / 20,
        "the fixture holds {} water tiles against {} open ones, so the refusal is barely reachable",
        flooded.len(),
        open.len()
    );
    assert!(
        open.len() > flooded.len() / 20,
        "the fixture holds {} open tiles against {} water ones, so it can hold no army",
        open.len(),
        flooded.len()
    );
}

#[test]
fn water_is_the_ground_that_admits_no_unit() {
    // The refusal comes from the kind. This states which kind, so a later
    // change to the kind set cannot quietly flood or drain the world without
    // a test noticing.
    let (world, _, flooded) = fixture();
    for address in flooded.iter().take(64) {
        assert_eq!(
            world.tile_kind(*address),
            Some(TileKind::Water),
            "the ground at {address:?} refused a unit and is not water"
        );
    }
}

#[test]
fn a_spawn_onto_water_is_refused() {
    let (mut world, _, flooded) = fixture();
    let wet = flooded[0];
    assert_eq!(
        world.spawn_soldier(wet, FactionId(0)),
        Err(SoldierError::TileImpassable(wet)),
        "the ground admitted a soldier onto water"
    );
}

#[test]
fn a_refusal_by_the_ground_is_not_a_refusal_by_the_extent() {
    // Two refusals, two variants. A caller that reports "outside the world"
    // for a tile in the middle of a lake sends the reader looking at the
    // wrong thing.
    let (mut world, _, flooded) = fixture();
    let outside = Axial::new(EXTENT as i32, 0);
    assert_eq!(
        world.spawn_soldier(outside, FactionId(0)),
        Err(SoldierError::TileOutsideWorld(outside))
    );
    assert_eq!(
        world.spawn_soldier(flooded[0], FactionId(0)),
        Err(SoldierError::TileImpassable(flooded[0]))
    );
}

#[test]
fn a_placement_onto_water_is_refused() {
    let (mut world, open, flooded) = fixture();
    let soldier = world
        .spawn_soldier(open[0], FactionId(0))
        .expect("the open tile admits a unit");
    let wet = flooded[0];
    assert_eq!(
        world.place_soldier(soldier, wet),
        Err(SoldierError::TileImpassable(wet)),
        "a placement walked a soldier into water"
    );
    assert_eq!(
        world.soldiers().address(soldier),
        Some(open[0]),
        "the refused placement moved the soldier anyway"
    );
}

#[test]
fn no_soldier_stands_on_water_after_a_run_of_frames() {
    // This is the test the item asked for. It drives the engine, not the
    // movement function, because the engine is what must honour the ground.
    let (mut world, open, _) = fixture();
    let mut kept = Vec::new();
    for (ordinal, address) in open.iter().enumerate().step_by(37) {
        kept.push(
            world
                .spawn_soldier(*address, FactionId((ordinal % 2) as u16))
                .expect("the open tile admits a unit"),
        );
    }
    assert!(
        kept.len() > 32,
        "the fixture spawned only {} soldiers",
        kept.len()
    );

    // The soldiers must start next to water, or no draw ever names a flooded
    // tile and the run proves nothing.
    let beside_water = kept
        .iter()
        .filter_map(|soldier| world.soldiers().address(*soldier))
        .filter(|here| {
            (0..6).any(|direction| {
                world
                    .grid()
                    .neighbour(*here, direction)
                    .is_some_and(|next| !world.admits_a_unit(next))
            })
        })
        .count();
    assert!(
        beside_water > 0,
        "no soldier started beside water, so no draw can name a flooded tile"
    );

    for frame in 0..FRAMES {
        world.step(2).expect("the step must run");
        for soldier in &kept {
            let here = world
                .soldiers()
                .address(*soldier)
                .expect("nothing despawned the soldier");
            assert!(
                world.admits_a_unit(here),
                "a soldier stood on water at {here:?} after frame {frame}"
            );
        }
        assert!(
            world.check_invariants(),
            "the world lost an invariant after frame {frame}"
        );
    }
}

#[test]
fn a_soldier_the_ground_refuses_holds_its_tile_and_stays_alive() {
    // The refusal must leave the soldier where it was. It must not despawn
    // it, must not move it to a default tile, and must not panic. A tile with
    // water on most sides is the strongest form of the case that this
    // generator offers: coherent ground makes no one-tile island.
    let (mut world, open, _) = fixture();
    let shore = open
        .iter()
        .copied()
        .find(|here| {
            // The tile lies away from the edge, so every refusal it meets
            // comes from the ground and never from the extent.
            here.q > 1
                && here.r > 1
                && here.q < EXTENT as i32 - 2
                && here.r < EXTENT as i32 - 2
                && (0..6)
                    .filter(|direction| {
                        world
                            .grid()
                            .neighbour(*here, *direction)
                            .is_some_and(|next| !world.admits_a_unit(next))
                    })
                    .count()
                    >= 3
        })
        .expect("the fixture holds an inland tile with water on three sides");

    let soldier = world
        .spawn_soldier(shore, FactionId(0))
        .expect("the shore admits a unit");
    let mut held = 0;
    let mut previous = shore;
    for frame in 0..FRAMES {
        world.step(1).expect("the step must run");
        let here = world
            .soldiers()
            .address(soldier)
            .expect("a refusal despawned the soldier");
        assert!(
            world.admits_a_unit(here),
            "the soldier stood on water at {here:?} after frame {frame}"
        );
        if here == previous {
            held += 1;
        }
        previous = here;
    }
    assert!(
        held > 0,
        "the soldier moved on every frame, so no draw was ever refused and the case is untested"
    );
}
