//! A spawn may carry a tile above its capacity, and the tile then drains.
//!
//! A spawn refuses impassable ground and it refuses a faction the world does
//! not hold. It does not read the occupancy of the tile, so a caller that
//! places a group may carry a tile above the capacity of its ground, and the
//! engine accepts it.[^1]
//!
//! Admission is the only rule that reads the capacity, and what it guarantees
//! is monotone. No tile gains a unit beyond its capacity. It is not that no
//! tile is ever above its capacity. Admission computes the room of a target by
//! subtracting the occupancy from the capacity, and the subtraction saturates,
//! so a tile above its capacity offers no room at all.[^2]
//!
//! **The occupancy is the count after the departures of the same tick.** A
//! frame runs several admission passes, and a departure releases room at the
//! end of a pass. A tile that stands above its capacity and then loses enough
//! units inside one frame falls below the capacity, and the later passes admit
//! against the lower count. Such a tile does take units in. The finding
//! register holds this correction.[^7]
//!
//! The fixture therefore keeps the tile above its capacity through the whole
//! frame. A tile that drained below the capacity inside the frame would admit
//! for a reason this suite is not about, and the assertion would measure the
//! fixture.
//!
//! **Neither determinism test can see an over-full tile.** The same placements
//! give the same result at every thread count, and the state hash matches its
//! golden file. Only a test that asserts the invariant can speak about it.[^3]
//!
//! # The fixture
//!
//! The suite builds a world that holds a tile above its capacity. A world
//! that never reaches a capacity supplies no extreme, so an assertion over it
//! never receives the input that would fail it.[^4] The fixture fills a patch
//! of open ground to the capacity of each tile, and then places three more
//! units on one tile of that patch.
//!
//! **A test that only watches the over-full tile fall is a guard, not
//! evidence.** It stays green when no unit ever asks for that tile, because
//! the assertion then receives no case.[^5] The suite therefore counts the
//! intents that named the over-full tile, and it fails when that count is
//! zero.
//!
//! The count needs the target that each unit drew. The engine draws it from
//! the counter-based generator, keyed on the system, the frame, the identity
//! and the draw. This file repeats that key, which is a second declaration
//! site for one value.[^6] A check holds the two sites together: every unit
//! that moved must have moved to the target this file predicted, and the
//! suite asserts that over every frame. The prediction cannot drift from the
//! engine without a red test.
//!
//! Every test drives the public interface of the core crate.
//!
//! # References
//!
//! [^1]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
//! [^2]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D2. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
//! [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`
//! [^5]: Findings register, FND-093. `docs/FINDINGS.md`
//! [^6]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
//! [^7]: Findings register, FND-110. `docs/FINDINGS.md`
//! [^8]: Findings register, FND-315. `docs/FINDINGS.md`

use std::collections::BTreeMap;

use cachette_core::rng;
use cachette_core::terrain::TileKind;
use cachette_core::{Axial, Entity, FactionId, World, WorldConfig, OPTION_COUNT};

/// The extent of the fixture world.
const EXTENT: u32 = 96;

/// The seed of the fixture world.
const SEED: u64 = 0x0cac_4e77_0023;

/// The number of units the fixture places on the over-full tile above the
/// capacity of its ground.
const OVER: u32 = 40;

/// The number of neighbours a tile has.
const NEIGHBOUR_COUNT: u64 = 6;

/// The draw index that the movement of a soldier uses for its direction.
///
/// This repeats a value the engine holds. The suite checks the repetition
/// against the engine on every frame.
const DRAW_MOVE_DIRECTION: u32 = 0;

/// The draw index that a soldier uses when the ground refuses its direction.
///
/// The engine answers a refused step with a second draw in the same system
/// and the same frame, so the second draw takes the next index. A unit that
/// only stayed put would stay put for ever, because every input to the
/// refused direction holds from one frame to the next. The module note above
/// cites the finding that measured it.
///
/// This repeats a value the engine holds, and the suite checks the repetition
/// against the engine on every frame.
const DRAW_MOVE_FALLBACK: u32 = 1;

/// Builds the fixture world.
fn world_of(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world");
    // A unit takes an intent at the interval its level 1 cell schedules. A
    // test about movement sets the interval to every tick.
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world
}

/// Returns the capacity of the ground at an address.
fn capacity_at(world: &World, address: Axial) -> u32 {
    world.tile_kind(address).map_or(0, TileKind::capacity)
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

/// Returns the addresses of a patch of open ground, in index order.
fn patch(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .filter(|address| address.q >= 8 && address.q < 20 && address.r >= 8 && address.r < 20)
        .filter(|address| world.admits_a_unit(*address))
        .collect()
}

/// Returns a tile of the patch whose six neighbours all admit a unit.
///
/// A tile with a closed neighbour would give the crowd fewer ways to ask for
/// it, and the suite needs the asking.
fn ringed_tile(world: &World) -> Axial {
    let grid = world.grid();
    patch(world)
        .into_iter()
        .find(|address| {
            (0..6).all(|direction| {
                grid.neighbour(*address, direction)
                    .is_some_and(|next| world.admits_a_unit(next))
            })
        })
        .expect("the patch holds a tile with six open neighbours")
}

/// Builds a world that holds one tile above the capacity of its ground.
///
/// Returns the world and the address of that tile.
fn over_filled(seed: u64) -> (World, Axial) {
    let mut world = world_of(seed);
    let ground = patch(&world);
    assert!(
        ground.len() >= 16,
        "the seed left only {} open tiles in the patch",
        ground.len()
    );
    let over = ringed_tile(&world);

    for address in &ground {
        for ordinal in 0..capacity_at(&world, *address) {
            world
                .spawn_soldier(*address, FactionId((ordinal % 2) as u16))
                .expect("the open tile admits a unit");
        }
    }
    // The tile is now at its capacity. These three placements carry it above
    // the capacity, and the engine accepts each one.
    for ordinal in 0..OVER {
        world
            .spawn_soldier(over, FactionId((ordinal % 2) as u16))
            .expect("a spawn does not read the capacity");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");

    (world, over)
}

#[test]
fn a_spawn_is_granted_on_a_tile_already_at_its_capacity() {
    // Decision D1, stated as a test. The tile is at the capacity of its
    // ground, and the spawn is granted anyway.
    let mut world = world_of(SEED);
    let address = ringed_tile(&world);
    let capacity = capacity_at(&world, address);
    assert!(capacity > 0, "the ground of the fixture admits no unit");

    for ordinal in 0..capacity {
        world
            .spawn_soldier(address, FactionId((ordinal % 2) as u16))
            .expect("the open tile admits a unit");
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
    assert_eq!(
        world.soldier_count_on(address).expect("the tile is inside"),
        capacity as usize,
        "the fixture did not reach the capacity, so the spawn below proves nothing",
    );

    let extra = world.spawn_soldier(address, FactionId(0));
    assert!(
        extra.is_ok(),
        "the spawn refused a tile at its capacity: {:?}",
        extra.err(),
    );

    world.rebuild_bridge(1).expect("the rebuild must succeed");
    assert_eq!(
        world.soldier_count_on(address).expect("the tile is inside"),
        capacity as usize + 1,
        "the unit was accepted and then stood nowhere",
    );
}

#[test]
fn a_spawn_still_refuses_ground_that_admits_no_unit() {
    // D1 removes the capacity refusal and nothing else. A test of the grant
    // alone would pass on a spawn that refused nothing at all.
    let mut world = world_of(SEED);
    let grid = world.grid();
    let closed = (0..grid.tile_count())
        .map(|index| Axial::new((index % grid.width()) as i32, (index / grid.width()) as i32))
        .find(|address| !world.admits_a_unit(*address))
        .expect("the fixture world holds ground that admits no unit");

    assert!(
        world.spawn_soldier(closed, FactionId(0)).is_err(),
        "the spawn placed a unit on ground that admits none",
    );
    let open = ringed_tile(&world);
    assert!(
        world.spawn_soldier(open, FactionId(9)).is_err(),
        "the spawn placed a unit of a faction the world does not hold",
    );
}

/// What one frame of the crowded run reported.
struct Frame {
    /// The number of intents that named the over-full tile.
    asked_for_the_over_full_tile: u32,
    /// The number of units that arrived on the over-full tile.
    arrived_at_the_over_full_tile: u32,
    /// The number of units that left the over-full tile.
    left_the_over_full_tile: u32,
    /// The number of units that moved, and whose target this file predicted.
    predicted: u32,
}

/// Runs one frame and checks the prediction of the direction against the
/// engine.
///
/// **A unit takes its direction from the exit field of its cell, and it falls
/// back to the keyed draw only where its cell holds no direction.**[^1] This
/// file therefore predicts both halves. It reads the field before the step,
/// because the step derives the field again at its own barrier and a unit acts
/// on the field that the last barrier left.[^2] It reads the option after the
/// step, because the choice pass writes the option inside the frame and before
/// the movement that reads it.
///
/// Returns what the frame reported. The caller asserts the invariant from the
/// occupancy it reads around the call.
///
/// # References
///
/// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decisions D1 and D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
/// [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
fn run_frame(world: &mut World, threads: usize, over: Axial) -> Frame {
    let grid = world.grid();
    let terrain = world.terrain();
    let seed = world.config().seed;
    let before: Vec<(Entity, Axial)> = world
        .soldiers()
        .iter()
        .filter_map(|soldier| Some((soldier, world.soldiers().address(soldier)?)))
        .collect();
    // The field as the last barrier left it. The step below replaces it.
    let field: Vec<Option<u8>> = {
        let exits = world.exit_field();
        (0..exits.cells().tile_count())
            .flat_map(|cell| {
                (0..OPTION_COUNT as u8)
                    .map(move |option| (cell, option))
                    .collect::<Vec<_>>()
            })
            .map(|(cell, option)| exits.exit(cell, option).expect("the entry exists"))
            .collect()
    };
    let layout = world.pyramid().layout();

    world.step(threads).expect("the step must run");
    let frame = world.tick().0;

    let mut asked = 0;
    let mut arrived = 0;
    let mut left = 0;
    let mut predicted = 0;
    for (soldier, was) in before {
        let Some(now) = world.soldiers().address(soldier) else {
            continue;
        };
        // A unit with no intent never asked for anything.
        let option = world.soldier_intent(soldier).flatten();
        let holds_an_intent = option.is_some();
        let steered = option.and_then(|option| {
            let tile = grid.index_of(was)?;
            let cell = layout.block_of_key(layout.key_of(tile)?);
            field[cell as usize * OPTION_COUNT + option as usize]
        });
        let direction = match steered {
            Some(direction) => direction as usize,
            None => rng::draw_below(
                seed,
                rng::SYSTEM_SOLDIER_MOVE,
                frame,
                soldier.to_bits(),
                DRAW_MOVE_DIRECTION,
                NEIGHBOUR_COUNT,
            ) as usize,
        };
        // The engine refuses a step off the world and a step onto ground
        // that admits no unit, and it answers either refusal with a second
        // keyed draw rather than by holding the unit still.[^8] A unit whose
        // second draw is refused as well does stay put for that frame, and
        // the next frame keys a different draw. This file repeats that rule,
        // because it repeats the key.[^6]
        let step_target = |here: Axial, direction: usize| -> Option<Axial> {
            let target = grid.neighbour(here, direction)?;
            terrain.kind(target)?.is_passable().then_some(target)
        };
        let target = match step_target(was, direction) {
            Some(target) => Some(target),
            None => {
                let again = rng::draw_below(
                    seed,
                    rng::SYSTEM_SOLDIER_MOVE,
                    frame,
                    soldier.to_bits(),
                    DRAW_MOVE_FALLBACK,
                    NEIGHBOUR_COUNT,
                ) as usize;
                step_target(was, again)
            }
        };
        if holds_an_intent && target == Some(over) && was != over {
            asked += 1;
        }
        if now == over && was != over {
            arrived += 1;
        }
        if was == over && now != over {
            left += 1;
        }
        if now != was {
            // The unit moved. It can only have moved to the target that the
            // exit field of its cell named, to the target of its draw where
            // the cell named none, or to the target of its fall-back draw
            // where the ground refused the first, so the prediction above and
            // the engine agree.
            assert_eq!(
                target,
                Some(now),
                "the unit moved to {now:?} and this file predicted {target:?}, \
                 so the draw key here no longer matches the engine",
            );
            predicted += 1;
        }
    }
    Frame {
        asked_for_the_over_full_tile: asked,
        arrived_at_the_over_full_tile: arrived,
        left_the_over_full_tile: left,
        predicted,
    }
}

/// Places units on one tile until it stands above the capacity of its ground.
///
/// The tile drains as its units depart, so a run that only over-filled it once
/// would hold the case for one frame. This restores the case before every
/// frame.
fn refill(world: &mut World, address: Axial) {
    let capacity = capacity_at(world, address);
    let mut standing = occupancy(world).get(&address).copied().unwrap_or(0);
    while standing < capacity + OVER {
        world
            .spawn_soldier(address, FactionId((standing % 2) as u16))
            .expect("a spawn does not read the capacity");
        standing += 1;
    }
    world.rebuild_bridge(1).expect("the rebuild must succeed");
}

#[test]
fn an_over_full_tile_admits_nobody_while_its_units_depart() {
    // Decision D2, stated as a test. The tile drains and never fills.
    //
    // The assertion is on the arrivals, not on the count. A count can fall
    // over a frame that admitted a unit, because more units left than
    // arrived, so a count hides the admission this test is about.
    let (mut world, over) = over_filled(SEED);
    let capacity = capacity_at(&world, over);

    let mut asked = 0;
    let mut left = 0;
    let mut predicted = 0;
    for frame in 0..16 {
        // The tile drains, so the case must be restored before each frame.
        refill(&mut world, over);
        let before = occupancy(&world).get(&over).copied().unwrap_or(0);
        assert_eq!(
            before,
            capacity + OVER,
            "the tile is not above its capacity, so frame {frame} is no case",
        );

        let report = run_frame(&mut world, 2, over);
        asked += report.asked_for_the_over_full_tile;
        left += report.left_the_over_full_tile;
        predicted += report.predicted;

        let after = occupancy(&world).get(&over).copied().unwrap_or(0);
        // Admission counts the arrivals of a tile against its occupancy after
        // the departures of the same tick. A tile that drained below its
        // capacity inside the frame is therefore no longer a case, and it
        // would admit for a reason this test is not about. The fixture places
        // enough units that the tile stays above its capacity throughout.
        assert!(
            after > capacity,
            "the tile fell to {after} inside frame {frame}, at or below the \
             {capacity} its ground admits, so the refusal was never the \
             reason nobody arrived",
        );
        assert_eq!(
            report.arrived_at_the_over_full_tile, 0,
            "the tile held {before} units, above the {capacity} its ground \
             admits, and it still took {} in on frame {frame}",
            report.arrived_at_the_over_full_tile,
        );
    }

    // Without these three, the assertion above is a guard rather than
    // evidence. It passes on a run where nobody asked for the tile.
    assert!(
        asked > 0,
        "no intent named the over-full tile over sixteen frames, so the \
         refusal was never exercised",
    );
    assert!(
        predicted > 0,
        "no unit moved, so the check that ties the draw to the engine ran on \
         nothing",
    );
    assert!(
        left > 0,
        "no unit left the over-full tile, so the drain is untested",
    );
}

#[test]
fn no_tile_rises_above_its_capacity_in_an_over_filled_world() {
    // The monotone invariant, over a world that holds a tile above its
    // capacity. A test that asserted the strong form, that no tile is ever
    // above its capacity, would fail on this world at frame zero.
    let (mut world, over) = over_filled(SEED);
    assert!(
        occupancy(&world)
            .iter()
            .any(|(address, count)| *count > capacity_at(&world, *address)),
        "no tile of the fixture is above its capacity, so the strong form \
         would have passed and this test measures nothing",
    );

    for frame in 0..16 {
        // The over-full tile drains, so the case must be restored before each
        // frame. A world that held the extreme for one frame would leave the
        // assertion measuring an ordinary world for the other fifteen.
        refill(&mut world, over);
        let before = occupancy(&world);
        run_frame(&mut world, 2, over);
        let after = occupancy(&world);

        for (address, count) in &after {
            let capacity = capacity_at(&world, *address);
            let was = before.get(address).copied().unwrap_or(0);
            assert!(
                *count <= capacity.max(was),
                "the tile {address:?} holds {count} after frame {frame}, \
                 above its capacity {capacity} and above the {was} it held before",
            );
        }
    }
}

#[test]
fn an_over_filled_world_gives_one_answer_at_every_thread_count() {
    // The over-fill must not become a source of nondeterminism. Admission
    // reads the occupancy of a target from the derived structure, and a tile
    // above its capacity is the case where the subtraction saturates.
    let expected = run_and_read(SEED, 6, 1);
    for threads in [2, 12] {
        assert_eq!(
            run_and_read(SEED, 6, threads),
            expected,
            "the positions differ at {threads} threads",
        );
    }
}

/// Runs the frames over the over-filled world and returns where each soldier
/// stands.
fn run_and_read(seed: u64, frames: u64, threads: usize) -> Vec<Axial> {
    let (mut world, _) = over_filled(seed);
    for _ in 0..frames {
        world.step(threads).expect("the step must run");
    }
    world
        .soldiers()
        .iter()
        .filter_map(|soldier| world.soldiers().address(soldier))
        .collect()
}
