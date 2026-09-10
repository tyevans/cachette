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
//! A world is one region of a stormy planet, so a run of frames ends some of
//! the soldiers it seats.[^6] A soldier a storm took says nothing about the
//! ground. The stepping test drops the soldiers the storm log names and no
//! others, and it holds a shore party on every frame so that the refusal stays
//! reachable.[^7]
//!
//! The tests see only the public crate API.[^5]
//!
//! # References
//!
//! [^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
//! [^2]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
//! [^3]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D2. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^5]: Testing policy. `docs/TESTING.md`
//! [^6]: Findings register, FND-744. `docs/FINDINGS.md`
//! [^7]: Findings register, FND-750. `docs/FINDINGS.md`

use cachette_core::{Axial, Entity, FactionId, SoldierError, TileKind, World, WorldConfig};

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
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
        ..WorldConfig::DEFAULT
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

/// Answers whether the storm log of the last step names the unit.
///
/// A world is one region of a stormy planet, so a run of frames ends units
/// that stand in the open.[^1] A soldier a storm took proves nothing about
/// the ground, and this suite drops exactly the units the log names. A loop
/// that dropped every unit the world no longer holds would pass against a run
/// that lost them all for another reason.[^2]
///
/// # References
///
/// [^1]: Findings register, FND-744. `docs/FINDINGS.md`
/// [^2]: Findings register, FND-750. `docs/FINDINGS.md`
fn a_storm_took(world: &World, unit: Entity) -> bool {
    world
        .units_lost_to_storms()
        .iter()
        .any(|lost| lost.unit == unit.to_bits())
}

/// Counts the neighbours of an address that the ground refuses a unit on.
///
/// A soldier away from every water tile can take no step the ground refuses,
/// so it exercises nothing this suite asserts.
fn water_neighbours(world: &World, here: Axial) -> usize {
    (0..6)
        .filter(|direction| {
            world
                .grid()
                .neighbour(here, *direction)
                .is_some_and(|next| !world.admits_a_unit(next))
        })
        .count()
}

/// Answers whether the ground refuses a unit on any neighbour of the address.
fn stands_beside_water(world: &World, here: Axial) -> bool {
    water_neighbours(world, here) > 0
}

/// The destination plane that the stepping test steers its company through.
const PLANE: u16 = 0;

/// The stride that picks the shore tiles the stepping test seats a soldier on.
///
/// The stride spreads the company over the shoreline of the whole world, so
/// no one lake decides the run.
const SHORE_STRIDE: usize = 7;

/// The fixture world holds enough of each ground for the suite to have a case.
///
/// A fixture of all open ground never offers a soldier a tile to be refused,
/// and every assertion of this file then passes without a case. A fixture of
/// all water holds no soldier at all.
#[test]
fn the_fixture_world_holds_both_water_and_open_ground() {
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

/// Water is the kind of ground that the refusal comes from.
///
/// The test states which kind, so a later change to the kind set cannot
/// quietly flood or drain the world without a test noticing.
#[test]
fn water_is_the_ground_that_admits_no_unit() {
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

/// Two refusals arrive as two variants.
///
/// A caller that reports "outside the world" for a tile in the middle of a
/// lake sends the reader looking at the wrong thing.
#[test]
fn a_refusal_by_the_ground_is_not_a_refusal_by_the_extent() {
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

/// The movement pass honours the ground on every frame of a run.
///
/// The test drives the engine, not the movement function, because the engine
/// is what must honour the ground.
///
/// **The company stands on the shoreline and is sent across the water.** A
/// company seated on open ground and left to itself steers inland, and it
/// takes a handful of steps from a shore tile over the whole run. The ground
/// then refuses almost nothing, and the assertion measures the fixture rather
/// than the engine.[^1] Every soldier here starts on a tile with water on two
/// sides, and the destination is the middle of a lake, so the exit of each
/// cell points at water on every frame.
///
/// **A storm ends a soldier that stands in the open, and that is not a
/// statement about the ground.**[^2] The loop drops the soldiers the storm log
/// names and no others, and it refuses a frame that leaves fewer than a
/// quarter of the company on the shoreline, so a run that lost the company
/// cannot pass.[^3]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.agents/rules/testing.md`
/// [^2]: Findings register, FND-744. `docs/FINDINGS.md`
/// [^3]: Findings register, FND-750. `docs/FINDINGS.md`
#[test]
fn no_soldier_stands_on_water_after_a_run_of_frames() {
    let (mut world, _, flooded) = fixture();
    let grid = world.grid();
    let shore: Vec<Axial> = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| world.admits_a_unit(*address) && water_neighbours(&world, *address) >= 2)
        .collect();
    let lake = flooded
        .iter()
        .copied()
        .find(|address| water_neighbours(&world, *address) == 6)
        .expect("the fixture holds a water tile with water on every side");

    let mut kept = Vec::new();
    for (ordinal, address) in shore.iter().enumerate().step_by(SHORE_STRIDE) {
        kept.push(
            world
                .spawn_soldier(*address, FactionId((ordinal % 2) as u16))
                .expect("the shore tile admits a unit"),
        );
    }
    assert!(
        kept.len() > 32,
        "the fixture seated only {} soldiers on the shoreline of {} shore tiles",
        kept.len(),
        shore.len()
    );
    world
        .send_units_to(&kept, &[lake], PLANE)
        .expect("the company takes a destination");

    let shoreline_floor = kept.len() / 4;
    let mut taken_by_a_storm: Vec<u64> = Vec::new();
    for frame in 0..FRAMES {
        world.step(2).expect("the step must run");
        taken_by_a_storm.extend(
            kept.iter()
                .filter(|soldier| a_storm_took(&world, **soldier))
                .map(|soldier| soldier.to_bits()),
        );
        let mut standing = 0;
        let mut on_the_shore = 0;
        for soldier in &kept {
            let Some(here) = world.soldiers().address(*soldier) else {
                assert!(
                    taken_by_a_storm.contains(&soldier.to_bits()),
                    "the soldier left the world after frame {frame} and no storm log named it"
                );
                continue;
            };
            assert!(
                world.admits_a_unit(here),
                "a soldier stood on water at {here:?} after frame {frame}"
            );
            standing += 1;
            if stands_beside_water(&world, here) {
                on_the_shore += 1;
            }
        }
        assert!(
            on_the_shore >= shoreline_floor,
            "{standing} soldiers of {} stood after frame {frame} and only {on_the_shore} of them \
             beside water, so the ground refused almost nothing on that frame",
            kept.len()
        );
        assert!(
            world.check_invariants(),
            "the world lost an invariant after frame {frame}"
        );
    }
}

/// A refusal by the ground leaves the soldier where it was.
///
/// The refusal must not despawn the soldier, must not move it to a default
/// tile, and must not panic. A tile with water on most sides is the strongest
/// form of the case that this generator offers, because coherent ground makes
/// no one-tile island. The tile lies away from the edge, so every refusal it
/// meets comes from the ground and never from the extent.
///
/// **A storm ends a soldier that stands in the open.**[^1] The loop asserts on
/// every frame that the storm log does not name this one, so a sky that took
/// it gives a failure that names the sky.
///
/// # References
///
/// [^1]: Findings register, FND-744. `docs/FINDINGS.md`
#[test]
fn a_soldier_the_ground_refuses_holds_its_tile_and_stays_alive() {
    let (mut world, open, _) = fixture();
    let shore = open
        .iter()
        .copied()
        .find(|here| {
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
        assert!(
            !a_storm_took(&world, soldier),
            "a storm ended the soldier on frame {frame}, so the run measures the sky \
             and not the ground"
        );
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
