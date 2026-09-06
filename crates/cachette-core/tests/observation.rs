//! A faction sees the ground its own units watch, and it remembers the rest.
//!
//! Every test here drives the world step. The step is what must run the
//! observation pass, so a test that called the pass itself would prove that
//! the pass works and not that anything reaches it.[^1]
//!
//! **Every fixture asserts that it produced the case the test needs.** A
//! world built from the demonstration seed is chosen to look right, not to
//! produce an edge value, so each fixture here states the distribution it
//! needs and fails when the world does not give it.[^2]
//!
//! # References
//!
//! [^1]: Testing rules, drive the real caller. `.agents/rules/testing.md`
//! [^2]: Testing rules, section 2a. `.agents/rules/testing.md`

use cachette_core::observation::BlockForm;
use cachette_core::{Axial, Entity, FactionId, SightRules, World, WorldConfig};

/// A world wide enough to hold two factions that never meet.
const TWO_SIDES: WorldConfig = WorldConfig {
    width: 96,
    height: 96,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 3,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// A world small enough for one unit to see every tile of it.
///
/// The whole world is one block of the lattice, and the largest distance
/// between two tiles of it is inside the sight ceiling. A faction that sees
/// the whole world therefore collapses its one block to the payload-free
/// form, which is the extreme the block container exists to reach.
const ONE_BLOCK: WorldConfig = WorldConfig {
    width: 16,
    height: 16,
    seed: 0x00c0_ffee_0123_4567,
    faction_count: 2,
    unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
};

/// The faction that watches in every fixture below.
const WATCHER: FactionId = FactionId(0);

/// The faction that stays away from the watcher.
const STRANGER: FactionId = FactionId(1);

/// The exponent that keeps a unit still.
///
/// A unit takes a movement intent at the interval its cell schedules. A long
/// interval stops a unit taking one inside a short test, so the test measures
/// the observation pass and not the movement pass.[^1]
///
/// # References
///
/// [^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
const KEEP_STILL: u32 = 12;

/// Builds a world in which no unit takes a movement intent.
fn a_still_world(config: WorldConfig) -> World {
    let mut world = World::new(config).expect("the configuration describes a world");
    world
        .set_choice_schedule(KEEP_STILL)
        .expect("the exponent is inside the range");
    world
}

/// Returns an address that admits a unit, near the one asked for.
///
/// The terrain comes from the seed, so the tile a test names may hold water.
/// The search is a spiral over the rings around the address, and it takes the
/// first tile of the lowest ring. The order is fixed, so two runs return one
/// answer.
fn ground_near(world: &World, wanted: Axial, bound: i32) -> Axial {
    for ring in 0..=bound {
        for column in -ring..=ring {
            for row in -ring..=ring {
                if column.abs().max(row.abs()) != ring {
                    continue;
                }
                let candidate = Axial::new(wanted.q + column, wanted.r + row);
                if world.admits_a_unit(candidate) {
                    return candidate;
                }
            }
        }
    }
    panic!("the fixture found no ground that admits a unit near {wanted:?}");
}

/// Puts a unit of one faction on a tile and returns its identity.
fn a_unit_at(world: &mut World, address: Axial, faction: FactionId) -> Entity {
    world
        .spawn_soldier(address, faction)
        .expect("the fixture places a unit on ground that admits one")
}

/// A faction sees the ground under its own unit, and the step is what says so.
#[test]
fn a_step_gives_a_faction_the_ground_its_unit_watches() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(20, 20), 8);

    // Before any unit exists, the faction sees nothing at all. This is the
    // reading that every later assertion is measured against.
    assert_eq!(
        world.faction_seen_now(WATCHER),
        0,
        "a faction with no unit sees nothing"
    );

    a_unit_at(&mut world, home, WATCHER);
    world.step(1).expect("the step runs");

    assert!(
        world.faction_sees_now(WATCHER, home),
        "a faction sees the tile its own unit stands on"
    );
    assert!(
        world.faction_seen_now(WATCHER) > 1,
        "a sight of more than zero steps reaches more than the tile under the unit"
    );
    assert!(
        world.faction_has_seen(WATCHER, home),
        "what a faction sees now it has also seen"
    );
}

/// A faction observes nothing where no unit of that faction has been.
#[test]
fn a_faction_observes_no_ground_its_units_never_neared() {
    let mut world = a_still_world(TWO_SIDES);
    let near = ground_near(&world, Axial::new(10, 10), 8);
    let far = ground_near(&world, Axial::new(80, 80), 8);

    // The fixture needs the two tiles far enough apart that no sight can
    // span them. The distance is asserted, so a world that placed them close
    // fails here rather than passing the test for the wrong reason.
    let reach = world.sight_rules().rounded();
    assert!(
        near.distance(far) > reach * 2,
        "the fixture must put the two tiles outside each other's sight"
    );

    a_unit_at(&mut world, near, WATCHER);
    a_unit_at(&mut world, far, STRANGER);
    world.step(1).expect("the step runs");

    assert!(
        world.faction_sees_now(STRANGER, far),
        "the fixture must give the stranger sight of its own ground"
    );
    assert!(
        !world.faction_sees_now(WATCHER, far),
        "a faction does not see ground that only another faction watches"
    );
    assert!(
        !world.faction_has_seen(WATCHER, far),
        "a faction has not seen ground no unit of it has ever neared"
    );
    assert!(
        !world.faction_has_seen(STRANGER, near),
        "the rule holds in both directions"
    );
}

/// A faction keeps in memory what it stops seeing.
#[test]
fn what_the_visible_layer_loses_the_remembered_layer_keeps() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(30, 30), 8);

    let unit = a_unit_at(&mut world, home, WATCHER);
    world.step(1).expect("the step runs");
    let seen_at_home = world.faction_seen_now(WATCHER);
    assert!(
        world.faction_sees_now(WATCHER, home),
        "the fixture must give the watcher sight of the tile first"
    );

    // The unit leaves the world. Nothing of the faction watches the tile
    // now, so the derived layer must lose it.
    assert!(world.despawn_soldier(unit), "the unit was alive");
    world.step(1).expect("the step runs");

    assert_eq!(
        world.faction_seen_now(WATCHER),
        0,
        "a faction with no live unit sees nothing"
    );
    assert!(
        !world.faction_sees_now(WATCHER, home),
        "a faction stops seeing ground the moment its last watcher goes"
    );
    assert!(
        world.faction_has_seen(WATCHER, home),
        "a faction remembers the ground it saw"
    );
    assert_eq!(
        world.faction_seen_ever(WATCHER),
        seen_at_home,
        "the remembered layer keeps every tile the visible layer held"
    );
}

/// A unit that marches adds ground to the memory and takes none away.
#[test]
fn a_unit_that_marches_grows_the_memory_and_never_shrinks_it() {
    let mut world = a_still_world(TWO_SIDES);
    let first = ground_near(&world, Axial::new(20, 20), 8);
    let second = ground_near(&world, Axial::new(60, 60), 8);
    let reach = world.sight_rules().rounded();
    assert!(
        first.distance(second) > reach * 2,
        "the fixture must put the two camps outside each other's sight"
    );

    let unit = a_unit_at(&mut world, first, WATCHER);
    world.step(1).expect("the step runs");
    let after_first = world.faction_seen_ever(WATCHER);
    assert!(
        !world.faction_has_seen(WATCHER, second),
        "the fixture must leave the second camp unseen"
    );

    world
        .place_soldier(unit, second)
        .expect("the second camp admits a unit");
    world.step(1).expect("the step runs");

    assert!(
        world.faction_sees_now(WATCHER, second),
        "the faction sees where its unit stands now"
    );
    assert!(
        !world.faction_sees_now(WATCHER, first),
        "the faction stops seeing where its unit stood before"
    );
    assert!(
        world.faction_has_seen(WATCHER, first),
        "the faction remembers where its unit stood before"
    );
    assert!(
        world.faction_seen_ever(WATCHER) > after_first,
        "a march adds ground to the memory"
    );
}

/// Ground that blocks sight hides what stands behind it.
#[test]
fn a_blocker_hides_the_ground_behind_it() {
    let world_reader = a_still_world(TWO_SIDES);
    let rules = world_reader.sight_rules();

    // The fixture needs an observer tile with a blocking neighbour and open
    // ground two steps beyond it, in one straight line. The search states
    // that distribution rather than trusting the demonstration world to hold
    // it. The scan takes the first case in address order, so it is fixed.
    let grid = world_reader.grid();
    let mut found = None;
    'search: for row in 0..grid.height() as i32 {
        for column in 0..grid.width() as i32 {
            let stand = Axial::new(column, row);
            if !world_reader.admits_a_unit(stand) {
                continue;
            }
            for direction in 0..6 {
                let Some(near) = grid.neighbour(stand, direction) else {
                    continue;
                };
                let Some(far) = grid.neighbour(near, direction) else {
                    continue;
                };
                let Some(near_kind) = world_reader.tile_kind(near) else {
                    continue;
                };
                if !rules.blocks(near_kind) {
                    continue;
                }
                found = Some((stand, near, far));
                break 'search;
            }
        }
    }
    let (stand, near, far) =
        found.expect("the fixture must find an observer with a blocking neighbour");

    let mut world = a_still_world(TWO_SIDES);
    a_unit_at(&mut world, stand, WATCHER);
    world.step(1).expect("the step runs");

    assert!(
        world.faction_sees_now(WATCHER, near),
        "a faction sees the blocker itself"
    );
    assert!(
        !world.faction_sees_now(WATCHER, far),
        "a faction does not see through ground that blocks sight"
    );
}

/// A faction that sees a whole block stores no payload for it.
#[test]
fn a_faction_that_sees_a_whole_block_stores_no_payload() {
    let mut world = a_still_world(ONE_BLOCK);
    // The rules widen the sight to the ceiling and let nothing block it. The
    // extreme this reaches is the payload-free form at the top of the
    // density range, which the sparse form must collapse to rather than pay
    // for.
    world.set_sight_rules(SightRules::new(16, 1, 16, 0));

    let grid = world.grid();
    let mut placed = 0;
    for row in 0..grid.height() as i32 {
        for column in 0..grid.width() as i32 {
            let address = Axial::new(column, row);
            if world.admits_a_unit(address) {
                a_unit_at(&mut world, address, WATCHER);
                placed += 1;
            }
        }
    }
    assert!(placed > 0, "the fixture must place at least one unit");
    world.step(1).expect("the step runs");

    let tiles = i64::from(grid.tile_count());
    assert_eq!(
        world.faction_seen_now(WATCHER),
        tiles,
        "the fixture must give the watcher sight of every tile of the world"
    );
    let layer = world
        .observation()
        .visible_layer(WATCHER)
        .expect("the watcher observed something");
    assert_eq!(
        layer.block(0),
        Some(&BlockForm::All),
        "a block the faction sees whole holds no payload"
    );
    assert_eq!(
        world
            .observation()
            .remembered_layer(WATCHER)
            .and_then(|remembered| remembered.block(0)),
        Some(&BlockForm::All),
        "the remembered layer collapses in the same way"
    );
}

/// The remembered layer changes the state hash.
///
/// Two worlds end this test with the same tick, the same unit, on the same
/// tile. They differ in one thing alone: one of them walked past a valley
/// first, and remembers it. The hash must say so.[^1]
///
/// # References
///
/// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D4. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
#[test]
fn a_memory_of_ground_changes_the_state_hash() {
    let mut travelled = a_still_world(TWO_SIDES);
    let mut stayed = a_still_world(TWO_SIDES);
    let valley = ground_near(&travelled, Axial::new(20, 20), 8);
    let camp = ground_near(&travelled, Axial::new(60, 60), 8);
    let reach = travelled.sight_rules().rounded();
    assert!(
        valley.distance(camp) > reach * 2,
        "the fixture must put the valley outside the sight of the camp"
    );

    // One world walks the valley first. The other starts at the camp. Both
    // run two ticks and both end with one unit standing at the camp.
    let walker = a_unit_at(&mut travelled, valley, WATCHER);
    travelled.step(1).expect("the step runs");
    travelled
        .place_soldier(walker, camp)
        .expect("the camp admits a unit");
    travelled.step(1).expect("the step runs");

    let sitter = a_unit_at(&mut stayed, camp, WATCHER);
    stayed.step(1).expect("the step runs");
    stayed
        .place_soldier(sitter, camp)
        .expect("the camp admits a unit");
    stayed.step(1).expect("the step runs");

    assert!(
        travelled.faction_has_seen(WATCHER, valley),
        "the fixture must leave one world with a memory of the valley"
    );
    assert!(
        !stayed.faction_has_seen(WATCHER, valley),
        "the fixture must leave the other world without one"
    );
    assert_eq!(
        travelled.faction_seen_now(WATCHER),
        stayed.faction_seen_now(WATCHER),
        "the fixture must leave the two worlds seeing the same ground now"
    );
    assert_ne!(
        travelled.state_hash(),
        stayed.state_hash(),
        "a memory of ground is stored state, so it enters the state hash"
    );
}

/// The sight rules change the state hash on the tick they are set.
///
/// The rules decide what the next rebuild writes, so a hash of the layers
/// alone would report the difference one tick after the cause.[^1]
///
/// # References
///
/// [^1]: ADR-0164, every stored value the step reads enters the state hash, decision D3. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
#[test]
fn the_sight_rules_change_the_state_hash() {
    let first = a_still_world(TWO_SIDES);
    let mut second = a_still_world(TWO_SIDES);
    assert_eq!(
        first.state_hash(),
        second.state_hash(),
        "two worlds built the same way hash the same"
    );

    second.set_sight_rules(SightRules::new(2, 1, 8, 0));
    assert_ne!(
        first.state_hash(),
        second.state_hash(),
        "the sight rules are a stored value the step reads, so they enter the hash"
    );
}

/// The observation pass gives one answer at every thread count.
#[test]
fn the_observation_pass_gives_one_answer_at_every_thread_count() {
    let answers: Vec<(u64, i64, i64)> = [1usize, 2, 12]
        .into_iter()
        .map(|threads| {
            let mut world = a_still_world(TWO_SIDES);
            // The fixture spreads the units across many blocks, because a
            // partition over one block would give every worker but one an
            // empty run and would measure nothing.
            for column in 0..6 {
                for row in 0..6 {
                    let wanted = Axial::new(8 + column * 14, 8 + row * 14);
                    let address = ground_near(&world, wanted, 6);
                    let faction = if (column + row) % 2 == 0 {
                        WATCHER
                    } else {
                        STRANGER
                    };
                    a_unit_at(&mut world, address, faction);
                }
            }
            for _ in 0..3 {
                world.step(threads).expect("the step runs");
            }
            (
                world.state_hash().finish(),
                world.faction_seen_now(WATCHER),
                world.faction_seen_ever(STRANGER),
            )
        })
        .collect();

    assert!(
        answers[0].1 > 0 && answers[0].2 > 0,
        "the fixture must give both factions ground to see"
    );
    assert_eq!(answers[0], answers[1], "one thread and two agree");
    assert_eq!(answers[0], answers[2], "one thread and twelve agree");
}

/// The level 1 masks name the factions that see each cell.
#[test]
fn a_cell_mask_names_the_factions_that_see_it() {
    let mut world = a_still_world(TWO_SIDES);
    let home = ground_near(&world, Axial::new(20, 20), 8);
    a_unit_at(&mut world, home, WATCHER);
    world.step(1).expect("the step runs");

    let layout = world.observation().layout();
    let key = layout
        .key_of(
            world
                .grid()
                .index_of(home)
                .expect("the tile is inside the world"),
        )
        .expect("the tile is inside the world");
    let cell = layout.block_of_key(key);

    assert!(
        world.cell_seen_now(cell).contains(WATCHER),
        "the mask names the faction that watches the cell"
    );
    assert!(
        !world.cell_seen_now(cell).contains(STRANGER),
        "the mask does not name a faction that watches nothing"
    );
    assert!(
        world.cell_seen_ever(cell).contains(WATCHER),
        "what a faction sees it has also seen"
    );
}

/// The stored population of every block agrees with its payload.
///
/// The population sits beside the payload, so a caller reads a cardinality
/// without walking the payload. Two copies of one fact need a check that
/// fails when they disagree.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
#[test]
fn every_stored_population_agrees_with_its_payload() {
    let mut world = a_still_world(TWO_SIDES);
    // The fixture reaches all four block forms. The dense cluster fills its
    // blocks past the threshold, the lone scout leaves a sparse block, and
    // the blocks nobody nears stay empty.
    for column in 0..8 {
        for row in 0..8 {
            let wanted = Axial::new(40 + column * 2, 40 + row * 2);
            let address = ground_near(&world, wanted, 4);
            a_unit_at(&mut world, address, WATCHER);
        }
    }
    let scout = ground_near(&world, Axial::new(8, 80), 6);
    a_unit_at(&mut world, scout, STRANGER);
    world.step(4).expect("the step runs");

    for faction in [WATCHER, STRANGER] {
        let observation = world.observation();
        assert!(
            observation
                .visible_layer(faction)
                .expect("the faction observed something")
                .populations_agree(),
            "the visible layer of a faction states one population for each block"
        );
        assert!(
            observation
                .remembered_layer(faction)
                .expect("the faction remembers something")
                .populations_agree(),
            "the remembered layer of a faction states one population for each block"
        );
    }
}
