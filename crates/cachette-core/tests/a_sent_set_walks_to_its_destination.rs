//! The control plane names a place, and a set of units walks to it.
//!
//! The control plane names a set of tiles and a set of units in one call. The
//! engine seeds one field at every one of those tiles and spreads a reach
//! outward. Every unit of the set reads the entry of its own cell and takes
//! one step.[^1]
//!
//! **No unit searches for a route.** A unit reads one entry. It reads no
//! neighbouring cell and it computes nothing from its own address toward a
//! seed, because the movement record forbids that.[^2]
//!
//! **A unit that the field cannot steer must not freeze.** A cell holds one
//! direction for a block of tiles, and every input to that direction holds
//! from one frame to the next, so a refusal repeats exactly. A unit against a
//! shoreline is the case that proved it.[^3]
//!
//! The tests drive the engine. A test that called the movement pass directly
//! would prove that the pass works and not that anything reaches it.[^4]
//!
//! **The fixture is built for the distribution these tests need.** It does not
//! copy the world of the demonstration binary.[^5] Each test states the
//! property of the ground it depends on, so a change to the generator fails
//! the fixture rather than the assertion.
//!
//! # References
//!
//! [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
//! [^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
//! [^3]: Findings register, FND-315. `docs/FINDINGS.md`
//! [^4]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^5]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::cohort::NeedRule;
use cachette_core::types::Fix32;
use cachette_core::{Axial, Entity, FactionId, TileIdx, World, WorldConfig};

/// The extent of every fixture world.
///
/// The extent covers several level 1 blocks in each direction, and it is wide
/// enough that the generator puts water in it.
const EXTENT: u32 = 256;

/// The extent of the world that holds a cell beyond the reach of the field.
///
/// The relaxation runs a fixed pass count, so a cell further than that count
/// from every seed holds no direction. This extent holds more cells in one row
/// than the pass count reaches, so the case exists whatever the generator put
/// in the world.[^1]
///
/// # References
///
/// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D5. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
const WIDE_EXTENT: u32 = 1024;

/// The seed of every fixture world.
///
/// It is the seed that the other fixtures of this project use, and its worlds
/// hold both water and open ground.
const SEED: u64 = 7;

/// The destination plane that every test names.
const PLANE: u16 = 0;

/// The number of frames a unit gets to leave the tile it started on.
///
/// The fall-back draw is uniform over the six neighbours, and a shoreline
/// tile may hold water on several of them, so one frame is not enough. This
/// count is a bound on a random walk and not a budget.[^1]
///
/// # References
///
/// [^1]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
const FRAMES: u64 = 32;

/// The number of frames a sent set gets to arrive.
///
/// A unit crosses one tile in each frame at best, and a cell is a block of
/// tiles, so crossing a few cells takes a few hundred frames. Ground that
/// refuses a step costs more, because the unit then takes a uniform draw.
/// This count is a bound on that walk and not a budget.[^1]
///
/// # References
///
/// [^1]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
const ARRIVAL_FRAMES: u64 = 4000;

/// The largest walk that the fixture accepts between a start and a
/// destination.
///
/// The bound stops the walk of the fixture, and it is not a property of the
/// engine. A start further than this is not used.
const WALK_BOUND: u32 = 512;

/// The number of hops that separates a start from a destination.
///
/// A start beside the destination would arrive on the first frame, and the
/// test would then pass without the field steering anything.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
const LEAST_HOPS: u32 = 2;

/// The largest number of hops that a start may sit from a destination.
const MOST_HOPS: u32 = 2;

/// Builds a fixture world at one extent, with the choice on every tick.
fn world(extent: u32) -> World {
    let mut world = World::new(WorldConfig {
        width: extent,
        height: extent,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    // The fixture holds two planes. The count is the fixture's, not the
    // world's, and it is what the refusal test names one above.
    world.set_destination_count(2);
    // **Nothing in the fixture starves.** These tests measure where a unit
    // walks over many frames, and a unit that died on the way would fail the
    // assertion for a reason that has nothing to do with the field. A decay
    // of zero holds every need full, so every unit stays fed.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    world.set_need_rule(
        NeedRule::new(
            Fix32::ZERO,
            Fix32::ZERO,
            Fix32::ZERO,
            Fix32::ZERO,
            NeedRule::DEFAULT.bound(),
        )
        .expect("no rate is below zero"),
    );
    world.rebuild_pyramid(1).expect("the rebuild must run");
    world
}

/// Returns the level 1 cell that covers one address.
fn cell_of(world: &World, address: Axial) -> u32 {
    let layout = world.pyramid().layout();
    let tile = world
        .grid()
        .index_of(address)
        .expect("the address is inside the world");
    layout.block_of_key(layout.key_of(tile).expect("the tile is inside the world"))
}

/// Returns the address of one cell of the lattice.
fn cell_address(world: &World, cell: u32) -> Axial {
    world
        .return_field()
        .cells()
        .address_of(TileIdx(cell))
        .expect("the cell is inside the lattice")
}

/// Returns the number of cells between an address and a seed cell.
///
/// The walk follows the direction of each cell in turn, which is what a unit
/// does one step at a time. It returns `None` when a cell on the way holds no
/// direction and is not the seed, which is the case that the field cannot
/// steer.
///
/// The walk is a property of the test and not of the engine. No unit walks a
/// chain of cells, because that would be the search the movement record
/// forbids.[^1]
///
/// # References
///
/// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
fn hops_to_seed(world: &World, from: Axial, seed_cell: u32) -> Option<u32> {
    let cells = world.return_field().cells();
    let mut cell = cell_of(world, from);
    let mut hops = 0;
    while cell != seed_cell {
        let here = cells.address_of(TileIdx(cell))?;
        let Some(Some(direction)) = world.destination_field().direction(PLANE, cell) else {
            return None;
        };
        let there = cells.neighbour(here, direction as usize)?;
        cell = cells.index_of(there)?.0;
        hops += 1;
        if hops > cells.tile_count() {
            return None;
        }
    }
    Some(hops)
}

/// Reports whether a tile and all six of its neighbours admit a unit.
///
/// **A start on a shoreline measures the fall-back and not the field.** The
/// ground refuses a step off such a tile on some of the six directions, so the
/// unit takes a uniform draw there and the walk that follows says little about
/// the field that steers it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn inland(world: &World, address: Axial) -> bool {
    if !world.admits_a_unit(address) {
        return false;
    }
    let grid = world.grid();
    (0..6).all(|direction| {
        grid.neighbour(address, direction)
            .is_some_and(|there| world.admits_a_unit(there))
    })
}

/// Returns the first open tile of the world, in ascending tile order.
///
/// The scan takes the first tile that fits, so the answer is fixed and does
/// not depend on how a caller walked the world.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn first_open_tile(world: &World) -> Axial {
    let grid = world.grid();
    for tile in 0..grid.tile_count() {
        let here = grid
            .address_of(TileIdx(tile))
            .expect("the tile is inside the world");
        if world.admits_a_unit(here) {
            return here;
        }
    }
    panic!("the fixture holds no ground that admits a unit");
}

/// Returns open tiles that sit a few cells from the destination.
///
/// The scan runs in ascending tile order and takes the first tiles that fit,
/// so the set is fixed.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn starts_a_few_cells_away(world: &World, seed_cell: u32, wanted: usize) -> Vec<Axial> {
    let grid = world.grid();
    let mut found = Vec::new();
    for tile in 0..grid.tile_count() {
        let here = grid
            .address_of(TileIdx(tile))
            .expect("the tile is inside the world");
        if !inland(world, here) {
            continue;
        }
        let Some(hops) = hops_to_seed(world, here, seed_cell) else {
            continue;
        };
        if !(LEAST_HOPS..=MOST_HOPS).contains(&hops) {
            continue;
        }
        // **The ground between the start and the destination must be open.**
        // A field at the pitch of a block says which way a block should go. It
        // cannot say how one unit gets around a lake in front of it, and a
        // record states that consequence.[^1] A start behind such a barrier
        // never arrives, and a test that used one would assert a promise the
        // engine does not make.[^2]
        //
        // [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, the consequences. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
        // [^2]: Findings register, FND-411. `docs/FINDINGS.md`
        if steps_along_the_field(world, here, seed_cell).is_none() {
            continue;
        }
        found.push(here);
        if found.len() == wanted {
            return found;
        }
    }
    panic!("the fixture holds no open tile between {LEAST_HOPS} and {MOST_HOPS} cells away");
}

/// Returns the number of tiles a unit walks when nothing refuses it.
///
/// The walk takes the direction of the cell the unit stands in and steps one
/// tile, which is what the movement pass does. It returns `None` when the
/// ground refuses a step, which is the case the field cannot route around.
///
/// The walk is a property of the test and not of the engine. No unit walks
/// ahead of itself, because that would be the search the movement record
/// forbids.[^1]
///
/// # References
///
/// [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
fn steps_along_the_field(world: &World, from: Axial, seed_cell: u32) -> Option<u32> {
    let grid = world.grid();
    let mut here = from;
    let mut steps = 0;
    while cell_of(world, here) != seed_cell {
        let cell = cell_of(world, here);
        let Some(Some(direction)) = world.destination_field().direction(PLANE, cell) else {
            return None;
        };
        let there = grid.neighbour(here, direction as usize)?;
        if !world.admits_a_unit(there) {
            return None;
        }
        here = there;
        steps += 1;
        if steps > WALK_BOUND {
            return None;
        }
    }
    Some(steps)
}

/// Sends a set of units to one tile, through the plane every test names.
fn send(world: &mut World, units: &[Entity], destination: Axial) {
    world
        .send_units_to(units, &[destination], PLANE)
        .expect("the destination, the units and the address must all be good");
}

#[test]
fn a_sent_set_walks_to_the_place_the_caller_named() {
    let mut world = world(EXTENT);
    let destination = first_open_tile(&world);
    // The seeds go in first with no units, so the fixture can read the field
    // and choose starts that the field can steer.
    send(&mut world, &[], destination);
    let seed_cell = cell_of(&world, destination);

    let starts = starts_a_few_cells_away(&world, seed_cell, 4);
    let units: Vec<Entity> = starts
        .iter()
        .map(|start| {
            world
                .spawn_soldier(*start, FactionId(0))
                .expect("the ground admits the unit")
        })
        .collect();

    // **The fixture must place every unit away from the destination.** A unit
    // that started in the destination cell would pass this test without the
    // field steering it anywhere.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    for start in &starts {
        assert_ne!(
            cell_of(&world, *start),
            seed_cell,
            "the unit at {start:?} starts in the destination cell"
        );
    }

    send(&mut world, &units, destination);

    let mut arrived = vec![false; units.len()];
    for _ in 0..ARRIVAL_FRAMES {
        world.step(1).expect("the step must run");
        for (index, unit) in units.iter().enumerate() {
            let here = world
                .soldiers()
                .address(*unit)
                .expect("the unit is still alive");
            if cell_of(&world, here) == seed_cell {
                arrived[index] = true;
            }
        }
        if arrived.iter().all(|reached| *reached) {
            break;
        }
    }

    for (index, reached) in arrived.iter().enumerate() {
        assert!(
            *reached,
            "the unit that started at {:?} did not reach the destination cell \
             in {ARRIVAL_FRAMES} frames",
            starts[index]
        );
    }
}

#[test]
fn a_sent_unit_the_ground_refuses_leaves_the_tile_it_started_on() {
    // The cell, the plane and the direction all hold from one frame to the
    // next, so a unit that only stayed put would stay put for ever. The
    // fall-back draw is keyed on the frame, so the unit takes a different
    // direction on the next frame.[^1] [^2]
    //
    // [^1]: Findings register, FND-315. `docs/FINDINGS.md`
    // [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    let mut world = world(EXTENT);
    let destination = first_open_tile(&world);
    send(&mut world, &[], destination);

    let start = a_tile_whose_destination_the_ground_refuses(&world);
    let unit = world
        .spawn_soldier(start, FactionId(0))
        .expect("the ground admits the unit");
    send(&mut world, &[unit], destination);

    let mut moved = false;
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        if world.soldiers().address(unit) != Some(start) {
            moved = true;
            break;
        }
    }
    assert!(
        moved,
        "the sent unit held tile {start:?} for {FRAMES} frames, so a refused \
         direction still freezes it"
    );
}

#[test]
fn a_sent_unit_beyond_the_reach_leaves_the_tile_it_started_on() {
    // The relaxation runs a fixed pass count, so a cell further than that
    // from every seed holds no direction. A unit there must fall back to the
    // keyed draw rather than stand still.[^1]
    //
    // [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    let mut world = world(WIDE_EXTENT);
    let destination = first_open_tile(&world);
    send(&mut world, &[], destination);
    let seed_cell = cell_of(&world, destination);

    let start = a_tile_the_field_cannot_steer(&world, seed_cell);
    let unit = world
        .spawn_soldier(start, FactionId(0))
        .expect("the ground admits the unit");
    send(&mut world, &[unit], destination);

    let mut moved = false;
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        if world.soldiers().address(unit) != Some(start) {
            moved = true;
            break;
        }
    }
    assert!(
        moved,
        "the sent unit at {start:?} holds no direction and held its tile for \
         {FRAMES} frames, so a cell the field cannot steer freezes a unit"
    );
}

#[test]
fn a_sent_unit_moves_before_it_has_chosen_anything() {
    // The choice pass writes an intent only on the frame that the cell of the
    // unit chooses, so a unit that has chosen nothing holds none. A sent unit
    // that waited for an intent would stand still until then.[^1]
    //
    // [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D2. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    let mut world = world(EXTENT);
    // The choice runs rarely, so the unit below holds no intent on the frames
    // that this test drives.
    world
        .set_choice_schedule(16)
        .expect("the exponent is inside the range");
    let destination = first_open_tile(&world);
    send(&mut world, &[], destination);
    let seed_cell = cell_of(&world, destination);

    let start = starts_a_few_cells_away(&world, seed_cell, 1)[0];
    let unit = world
        .spawn_soldier(start, FactionId(0))
        .expect("the ground admits the unit");

    // **The fixture must give the unit no intent.** A unit that had chosen
    // would move whatever this test asserts, and the assertion would then
    // measure the choice pass.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    assert_eq!(
        world.soldiers().intent(unit),
        Some(None),
        "the unit already holds an intent, so this test cannot fail"
    );

    send(&mut world, &[unit], destination);
    let mut moved = false;
    for _ in 0..FRAMES {
        world.step(1).expect("the step must run");
        if world.soldiers().address(unit) != Some(start) {
            moved = true;
            break;
        }
    }
    assert!(
        moved,
        "the sent unit at {start:?} holds no intent and never moved, so an \
         order waits for a choice that the caller did not ask for"
    );
}

#[test]
fn the_order_the_caller_names_the_seeds_in_does_not_reach_the_field() {
    // The seed set is a set. Two calls that name one set in two orders must
    // derive one field.[^1]
    //
    // [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    let mut ascending = world(EXTENT);
    let mut descending = world(EXTENT);
    let seeds = three_open_tiles(&ascending);
    let mut reversed = seeds.clone();
    reversed.reverse();
    assert_ne!(seeds, reversed, "the fixture named one tile three times");

    ascending
        .send_units_to(&[], &seeds, PLANE)
        .expect("the addresses are inside the world");
    descending
        .send_units_to(&[], &reversed, PLANE)
        .expect("the addresses are inside the world");

    assert_eq!(
        ascending.state_hash().finish(),
        descending.state_hash().finish(),
        "the order of the seeds reached the world state"
    );
    for cell in 0..ascending.return_field().cells().tile_count() {
        let address = cell_address(&ascending, cell);
        let _ = address;
        assert_eq!(
            ascending.destination_field().direction(PLANE, cell),
            descending.destination_field().direction(PLANE, cell),
            "cell {cell} holds two directions for one set of seeds"
        );
    }
}

#[test]
fn stopping_an_order_gives_the_choice_back() {
    let mut world = world(EXTENT);
    let destination = first_open_tile(&world);
    let start = first_open_tile(&world);
    let unit = world
        .spawn_soldier(start, FactionId(0))
        .expect("the ground admits the unit");

    assert_eq!(world.sent_to(unit), Some(None));
    send(&mut world, &[unit], destination);
    assert_eq!(world.sent_to(unit), Some(Some(PLANE)));
    world
        .stop_sending(&[unit])
        .expect("the identity names a live unit");
    assert_eq!(world.sent_to(unit), Some(None));
}

#[test]
fn a_send_refuses_a_destination_the_world_does_not_hold() {
    let mut world = world(EXTENT);
    let destination = first_open_tile(&world);
    let outside = world.destination_count();
    assert!(
        world.send_units_to(&[], &[destination], outside).is_err(),
        "the world took an order for a destination it does not hold"
    );
}

#[test]
fn a_sent_run_gives_one_answer_at_any_thread_count() {
    // A determinism test must compare a run against another run, and it must
    // run at more than one thread count.[^1] [^2]
    //
    // [^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    // [^2]: Testing rules, section 1. `.claude/rules/testing.md`
    let single = sent_run(1);
    for threads in [2, 12] {
        assert_eq!(
            single,
            sent_run(threads),
            "a sent set gives one answer at 1 thread and another at {threads}"
        );
    }
}

/// Runs the same sent scenario at one thread count and returns the state hash.
fn sent_run(threads: usize) -> u64 {
    let mut world = world(EXTENT);
    let destination = first_open_tile(&world);
    send(&mut world, &[], destination);
    let seed_cell = cell_of(&world, destination);
    let starts = starts_a_few_cells_away(&world, seed_cell, 4);
    let units: Vec<Entity> = starts
        .iter()
        .map(|start| {
            world
                .spawn_soldier(*start, FactionId(0))
                .expect("the ground admits the unit")
        })
        .collect();
    send(&mut world, &units, destination);
    for _ in 0..8 {
        world.step(threads).expect("the step must run");
    }
    world.state_hash().finish()
}

/// Returns three open tiles that sit in three different cells.
///
/// The scan runs in ascending tile order, so the answer is fixed.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn three_open_tiles(world: &World) -> Vec<Axial> {
    let grid = world.grid();
    let mut found: Vec<Axial> = Vec::new();
    for tile in 0..grid.tile_count() {
        let here = grid
            .address_of(TileIdx(tile))
            .expect("the tile is inside the world");
        if !world.admits_a_unit(here) {
            continue;
        }
        if found
            .iter()
            .any(|taken| cell_of(world, *taken) == cell_of(world, here))
        {
            continue;
        }
        found.push(here);
        if found.len() == 3 {
            return found;
        }
    }
    panic!("the fixture holds fewer than three cells with open ground");
}

/// Returns a tile whose cell holds a direction that the ground refuses.
///
/// This is the shape that froze a unit against a shoreline. The cell holds a
/// direction, and the tile in front of one unit of that block is water.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-315. `docs/FINDINGS.md`
fn a_tile_whose_destination_the_ground_refuses(world: &World) -> Axial {
    let grid = world.grid();
    for tile in 0..grid.tile_count() {
        let here = grid
            .address_of(TileIdx(tile))
            .expect("the tile is inside the world");
        if !world.admits_a_unit(here) {
            continue;
        }
        let Some(Some(direction)) = world.destination_direction(PLANE, here) else {
            continue;
        };
        let refused = grid
            .neighbour(here, direction as usize)
            .is_none_or(|target| !world.admits_a_unit(target));
        if refused {
            return here;
        }
    }
    panic!("the fixture holds no tile whose destination direction the ground refuses");
}

/// Returns an open tile whose cell holds no direction and is not a seed.
///
/// The tile is either beyond the reach of the relaxation or cut off from every
/// seed by ground that admits nobody. Both are cases that the field cannot
/// steer, and a unit at either must still move.[^1]
///
/// # References
///
/// [^1]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
fn a_tile_the_field_cannot_steer(world: &World, seed_cell: u32) -> Axial {
    let grid = world.grid();
    for tile in 0..grid.tile_count() {
        let here = grid
            .address_of(TileIdx(tile))
            .expect("the tile is inside the world");
        if !world.admits_a_unit(here) {
            continue;
        }
        if cell_of(world, here) == seed_cell {
            continue;
        }
        if world.destination_direction(PLANE, here) == Some(None) {
            return here;
        }
    }
    panic!("the fixture holds no open tile that the field cannot steer");
}
