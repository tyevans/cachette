//! Movement takes its direction from the per-cell exit field.
//!
//! The engine derives one exit direction for each level 1 cell and each
//! option. A unit reads the entry of its own cell and its own option, and it
//! steps to the neighbouring tile in that direction. No unit scores a
//! neighbouring cell of its own.[^1]
//!
//! The tests drive the engine. A test that called the derivation directly
//! would prove that the derivation works and not that anything reaches
//! it.[^2]
//!
//! **The fixture is built for the distribution these tests need.** It does not
//! copy the world of the demonstration binary, because that world is chosen to
//! look right and not to produce an extreme.[^3] The value under test is the
//! one that a test can set exactly: the number of units for each open tile of
//! a cell. A test places units, so it names the ranking of the neighbours of a
//! cell rather than accepting whatever the ground happened to generate.
//!
//! # References
//!
//! [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
//! [^2]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^3]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::choose;
use cachette_core::hex::NEIGHBOURS;
use cachette_core::{Axial, Entity, FactionId, Fix32, World, WorldConfig};

/// The extent of every fixture world.
///
/// The extent covers several level 1 blocks in each direction, so a cell in
/// the middle of it has all six neighbours inside the lattice.
const EXTENT: u32 = 256;

/// The seed of every fixture world.
///
/// Each test asserts the property of the ground that it depends on, so a
/// change to the generator fails the fixture rather than the assertion.
const SEED: u64 = 7;

/// The option index of the row that scores the units of a cell.
///
/// The row is the one whose cell value a test can set exactly, by placing
/// units. The index is the tie-break position of the row.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
const JOIN: u8 = 4;

/// The thread counts that the equivalence test runs at.
const THREAD_COUNTS: [usize; 3] = [1, 2, 12];

/// Builds the fixture world, with the choice on every tick.
fn world() -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed: SEED,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
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

/// Returns a cell whose six neighbours all lie inside the lattice and all hold
/// ground that admits a unit.
///
/// The fixture needs that shape, because a test sets the value of a
/// neighbouring cell by placing units on it, and a cell of open water takes
/// none. The scan runs in ascending cell order and takes the first cell that
/// fits, so the answer is fixed and does not depend on how a caller walked the
/// lattice.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
fn middle_cell(world: &World) -> u32 {
    let cells = world.exit_field().cells();
    for cell in 0..cells.tile_count() {
        let here = cells
            .address_of(cachette_core::TileIdx(cell))
            .expect("the cell is inside the lattice");
        if open_addresses_of(world, cell).is_empty() {
            continue;
        }
        let usable = (0..NEIGHBOURS.len()).all(|direction| {
            cells
                .neighbour(here, direction)
                .and_then(|there| cells.index_of(there))
                .is_some_and(|there| !open_addresses_of(world, there.0).is_empty())
        });
        if usable {
            return cell;
        }
    }
    panic!("the fixture holds no cell whose six neighbours all admit a unit");
}

/// Returns the cell that lies in one direction from another.
fn neighbour_cell(world: &World, cell: u32, direction: usize) -> u32 {
    let cells = world.exit_field().cells();
    let here = cells
        .address_of(cachette_core::TileIdx(cell))
        .expect("the cell is inside the lattice");
    let there = cells
        .neighbour(here, direction)
        .expect("the neighbour is inside the lattice");
    cells
        .index_of(there)
        .expect("the neighbour is inside the lattice")
        .0
}

/// Returns every address of one cell that admits a unit, in index order.
fn open_addresses_of(world: &World, cell: u32) -> Vec<Axial> {
    let layout = world.pyramid().layout();
    let edge = layout.block_edge();
    let first_column = (cell % layout.blocks_wide()) * edge;
    let first_row = (cell / layout.blocks_wide()) * edge;
    let mut found = Vec::new();
    for row in first_row..first_row + edge {
        for column in first_column..first_column + edge {
            let address = Axial::new(column as i32, row as i32);
            if world.admits_a_unit(address) {
                found.push(address);
            }
        }
    }
    found
}

/// Puts the given number of units on every open tile of one cell.
///
/// The value that the `join` row reads is the unit count divided by the open
/// tile count, so a cell filled at this rate reads exactly the rate. The test
/// therefore names the ranking of two cells and does not compute it.
fn stock(world: &mut World, cell: u32, for_each_open_tile: u32) -> Vec<Entity> {
    let addresses = open_addresses_of(world, cell);
    assert!(
        !addresses.is_empty(),
        "cell {cell} holds no ground that admits a unit"
    );
    let mut placed = Vec::new();
    for address in addresses {
        for _ in 0..for_each_open_tile {
            placed.push(
                world
                    .spawn_soldier(address, FactionId(0))
                    .expect("the open tile admits the unit"),
            );
        }
    }
    placed
}

/// Returns the value that the `join` row reads from one cell.
fn reading(world: &World, cell: u32) -> Fix32 {
    world
        .pyramid()
        .cell(cell)
        .expect("the cell is inside the level")
        .units_for_each_open_tile()
        .expect("the cell covers open ground")
}

/// Returns every entry of the exit field, in cell order then option order.
fn snapshot(world: &World) -> Vec<Option<u8>> {
    let field = world.exit_field();
    let mut entries = Vec::new();
    for cell in 0..field.cells().tile_count() {
        for option in 0..choose::OPTION_COUNT as u8 {
            entries.push(field.exit(cell, option).expect("the entry exists"));
        }
    }
    entries
}

#[test]
fn a_cell_points_at_the_neighbour_that_holds_the_most() {
    let mut world = world();
    let middle = middle_cell(&world);
    let target = neighbour_cell(&world, middle, 3);
    stock(&mut world, target, 2);
    let target = neighbour_cell(&world, middle, 1);
    stock(&mut world, target, 1);
    world.rebuild_pyramid(1).expect("the rebuild must run");

    // The fixture holds the contrast the assertion needs.
    let high = reading(&world, neighbour_cell(&world, middle, 3));
    let low = reading(&world, neighbour_cell(&world, middle, 1));
    assert!(
        high > low,
        "the fixture holds no contrast: {low:?} {high:?}"
    );
    assert_eq!(reading(&world, middle), Fix32::ZERO);

    assert_eq!(
        world.exit_field().exit(middle, JOIN),
        Some(Some(3)),
        "the cell did not point at the neighbour that holds the most"
    );
}

#[test]
fn a_change_to_one_neighbour_changes_the_direction() {
    // This is the test that the register asks for against every value the work
    // writes into state: change the value, and the decision must change.[^1]
    //
    // [^1]: Decisions register, DEC-074. `docs/DECISIONS.md`
    let mut world = world();
    let middle = middle_cell(&world);
    let far = neighbour_cell(&world, middle, 3);
    let near = neighbour_cell(&world, middle, 1);
    stock(&mut world, far, 2);
    stock(&mut world, near, 1);
    world.rebuild_pyramid(1).expect("the rebuild must run");
    assert_eq!(world.exit_field().exit(middle, JOIN), Some(Some(3)));

    // One neighbour rises above the other. Nothing else changes.
    stock(&mut world, near, 2);
    world.rebuild_pyramid(1).expect("the rebuild must run");
    assert!(reading(&world, near) > reading(&world, far));
    assert_eq!(
        world.exit_field().exit(middle, JOIN),
        Some(Some(1)),
        "the direction did not follow the value that the option reads"
    );
}

#[test]
fn the_lowest_direction_index_wins_a_tie() {
    let mut world = world();
    let middle = middle_cell(&world);
    let low = neighbour_cell(&world, middle, 0);
    let high = neighbour_cell(&world, middle, 4);
    stock(&mut world, low, 2);
    stock(&mut world, high, 2);
    world.rebuild_pyramid(1).expect("the rebuild must run");

    assert_eq!(
        reading(&world, low),
        reading(&world, high),
        "the fixture holds no tie"
    );
    assert_eq!(
        world.exit_field().exit(middle, JOIN),
        Some(Some(0)),
        "a tie went to a direction other than the lowest index"
    );
}

#[test]
fn a_cell_that_no_neighbour_beats_holds_no_direction() {
    let mut world = world();
    let middle = middle_cell(&world);
    stock(&mut world, middle, 2);
    world.rebuild_pyramid(1).expect("the rebuild must run");

    for direction in 0..NEIGHBOURS.len() {
        let there = neighbour_cell(&world, middle, direction);
        assert_eq!(reading(&world, there), Fix32::ZERO);
    }
    assert_eq!(
        world.exit_field().exit(middle, JOIN),
        Some(None),
        "a cell that no neighbour beats holds a direction"
    );
}

#[test]
fn a_unit_in_a_cell_with_no_direction_still_moves() {
    let mut world = world();
    let middle = middle_cell(&world);
    // The whole world is empty except this cell, so no neighbour beats it and
    // the field leaves the cell without a direction. The unit must still take
    // the uniform draw.
    let units = stock(&mut world, middle, 1);
    world.rebuild_pyramid(1).expect("the rebuild must run");
    assert_eq!(world.exit_field().exit(middle, JOIN), Some(None));
    only(&mut world, JOIN, Fix32::ONE);

    let before: Vec<Axial> = units
        .iter()
        .map(|unit| world.soldiers().address(*unit).expect("alive"))
        .collect();
    world.step(1).expect("the step must run");
    let after: Vec<Axial> = units
        .iter()
        .map(|unit| world.soldiers().address(*unit).expect("alive"))
        .collect();
    assert!(
        before.iter().zip(&after).any(|(here, there)| here != there),
        "no unit moved, so the uniform draw does not reach a cell with no exit"
    );
}

/// Puts every weight on one option and none on the others.
fn only(world: &mut World, option: u8, weight: Fix32) {
    for index in 0..choose::OPTION_COUNT as u8 {
        world
            .set_option_weight(index, Fix32::ZERO)
            .expect("the index is inside the set");
    }
    world
        .set_option_weight(option, weight)
        .expect("the index is inside the set");
}

/// Returns a direction that no option other than `join` points at from one
/// cell.
///
/// **The fixture must make the option matter.** A direction that two options
/// share proves nothing about which entry the step read, and a test built on
/// one stays green when the option column is pinned to a constant. The pin has
/// to reach the value the consumer reads.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-183. `docs/FINDINGS.md`
fn direction_only_join_points_at(world: &World, cell: u32) -> usize {
    let taken: Vec<Option<u8>> = (0..choose::OPTION_COUNT as u8)
        .filter(|option| *option != JOIN)
        .map(|option| {
            world
                .exit_field()
                .exit(cell, option)
                .expect("the entry exists")
        })
        .collect();
    (0..NEIGHBOURS.len())
        .find(|direction| {
            let candidate = Some(*direction as u8);
            !taken.contains(&candidate)
                && !open_addresses_of(world, neighbour_cell(world, cell, *direction)).is_empty()
        })
        .expect("every direction is taken by another option")
}

#[test]
fn the_step_moves_a_unit_in_the_direction_its_cell_holds() {
    // The test drives the step. It reads the entry the engine derived, and it
    // names the tile that the unit must reach.[^1]
    //
    // [^1]: Testing rules, section 5. `.claude/rules/testing.md`
    let mut world = world();
    let middle = middle_cell(&world);
    let direction = direction_only_join_points_at(&world, middle);
    let target = neighbour_cell(&world, middle, direction);
    stock(&mut world, target, 2);
    stock(&mut world, middle, 1);
    only(&mut world, JOIN, Fix32::ONE);

    // The watcher stands away from the block edge, and the tile it must reach
    // is open ground, so nothing but the direction decides where it lands.
    let open = open_addresses_of(&world, middle);
    let start = *open
        .iter()
        .find(|address| {
            let there = address.add(NEIGHBOURS[direction]);
            world.admits_a_unit(there) && cell_of(&world, there) == middle
        })
        .expect("the cell holds a tile whose neighbour in this direction is open");
    let watcher = world
        .spawn_soldier(start, FactionId(0))
        .expect("the open tile admits a unit");
    world.rebuild_pyramid(1).expect("the rebuild must run");

    // The fixture makes the option decide the answer. No other option points
    // this way, so a step that read a different entry lands elsewhere.
    assert_eq!(
        world.exit_field().exit(middle, JOIN),
        Some(Some(direction as u8))
    );
    for option in 0..choose::OPTION_COUNT as u8 {
        if option == JOIN {
            continue;
        }
        assert_ne!(
            world.exit_field().exit(middle, option),
            Some(Some(direction as u8)),
            "option {option} points the same way, so the fixture does not test the option"
        );
    }

    world.step(1).expect("the step must run");
    assert_eq!(
        world.soldier_intent(watcher).expect("alive"),
        Some(JOIN),
        "the watcher chose an option other than the one under test"
    );
    assert_eq!(
        world.soldiers().address(watcher),
        Some(start.add(NEIGHBOURS[direction])),
        "the unit did not step in the direction that its cell holds"
    );
}

#[test]
fn the_field_is_the_same_at_every_thread_count() {
    let mut fields = Vec::new();
    for threads in THREAD_COUNTS {
        let mut world = world();
        let middle = middle_cell(&world);
        let target = neighbour_cell(&world, middle, 3);
        stock(&mut world, target, 2);
        stock(&mut world, middle, 1);
        only(&mut world, JOIN, Fix32::ONE);
        for _ in 0..3 {
            world.step(threads).expect("the step must run");
        }
        fields.push(snapshot(&world));
    }
    assert!(
        fields[0].iter().any(Option::is_some),
        "the fixture holds no direction anywhere, so the comparison proves nothing"
    );
    assert_eq!(fields[0], fields[1]);
    assert_eq!(fields[0], fields[2]);
}

#[test]
fn deriving_the_field_twice_gives_one_answer() {
    // The field carries nothing between two derivations. A field that
    // accumulated would differ on the second pass over one level 1.[^1]
    //
    // [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
    let mut world = world();
    let middle = middle_cell(&world);
    let target = neighbour_cell(&world, middle, 3);
    stock(&mut world, target, 2);
    let target = neighbour_cell(&world, middle, 1);
    stock(&mut world, target, 1);
    world.rebuild_pyramid(1).expect("the rebuild must run");
    let first = snapshot(&world);
    world.rebuild_pyramid(1).expect("the rebuild must run");
    assert!(first.iter().any(Option::is_some));
    assert_eq!(first, snapshot(&world), "the field carried something");
}

#[test]
fn the_public_rebuild_derives_the_field_that_the_step_derives() {
    // Two paths rebuild level 1: the barrier of a step, and the public rebuild
    // that a caller runs outside a frame. A field left behind by one of them
    // would be a stale value that nothing fails on.[^1]
    //
    // [^1]: Findings register, FND-029. `docs/FINDINGS.md`
    let mut stepped = world();
    let middle = middle_cell(&stepped);
    let target = neighbour_cell(&stepped, middle, 3);
    stock(&mut stepped, target, 2);
    stock(&mut stepped, middle, 1);
    only(&mut stepped, JOIN, Fix32::ONE);
    stepped.step(1).expect("the step must run");

    // The same world state, reached by the public rebuild instead. The step
    // moves units, so the comparison runs over the world the step left.
    let mut rebuilt = stepped.clone();
    rebuilt.rebuild_pyramid(1).expect("the rebuild must run");
    assert!(snapshot(&stepped).iter().any(Option::is_some));
    assert_eq!(
        snapshot(&stepped),
        snapshot(&rebuilt),
        "the two rebuild paths left different fields"
    );
}

#[test]
fn a_world_that_has_never_stepped_holds_the_field() {
    // Building a world rebuilds level 1, so it derives the field too. A world
    // that answered nothing here would hand a stale entry to the first frame.
    let mut world = world();
    let middle = middle_cell(&world);
    let target = neighbour_cell(&world, middle, 3);
    stock(&mut world, target, 2);
    let built = world.clone();
    world.rebuild_pyramid(1).expect("the rebuild must run");
    assert_eq!(
        snapshot(&built).len(),
        snapshot(&world).len(),
        "the field of a new world covers a different lattice"
    );
}
