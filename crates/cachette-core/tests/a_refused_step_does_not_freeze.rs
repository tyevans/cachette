//! The ground may refuse the exit of a cell, and the unit must still move.
//!
//! The exit field holds one direction for each level 1 cell and each
//! option.[^1] A cell covers a block of tiles, and the ground under one unit
//! of that block may refuse the direction the block holds. That refusal is
//! not a one-frame accident: the cell, the option and the direction all hold
//! from one frame to the next, so a unit that only stayed put would stay put
//! for ever. A unit against a shoreline is the case that showed it.[^2]
//!
//! The tests drive the engine. A test that called the movement pass directly
//! would prove that the pass works and not that anything reaches it.[^3]
//!
//! **The fixture is built for the distribution these tests need.** It does not
//! copy the world of the demonstration binary.[^4] Each test states the
//! property of the ground it depends on, so a change to the generator fails
//! the fixture rather than the assertion.
//!
//! # References
//!
//! [^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
//! [^2]: Findings register, FND-315. `docs/FINDINGS.md`
//! [^3]: Testing rules, section 5. `.claude/rules/testing.md`
//! [^4]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::choose;
use cachette_core::hex::NEIGHBOURS;
use cachette_core::{Axial, FactionId, TileIdx, World, WorldConfig};

/// The extent of every fixture world.
///
/// The extent covers several level 1 blocks in each direction, and it is wide
/// enough that the generator puts water in it.
const EXTENT: u32 = 256;

/// The seed of the world that holds a cell ranking closed ground above itself.
///
/// **The seed is the fixture, and it was measured rather than chosen.** A cell
/// of open water outranks the cell beside it only on the mean height row, and
/// only where the water the source cell holds is deeper on average than the
/// water the closed cell holds. That is a tail of the generator, not the
/// typical case: a sweep of forty seeds found it in eleven of them, and the
/// seed this project uses elsewhere is one of the twenty-nine that miss it.
/// A fixture built from the ordinary world would assert against a case the
/// engine never reaches.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
const CLOSED_NEIGHBOUR_SEED: u64 = 4;

/// The seed of the world that holds a tile whose cell exit the ground refuses.
///
/// A cell exit that one tile of the block cannot take is the ordinary case,
/// not a tail, so this is the seed the other fixtures of this project use.
const REFUSED_SEED: u64 = 7;

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

/// Builds a fixture world at one seed, with the choice on every tick.
fn world(seed: u64) -> World {
    let mut world = World::new(WorldConfig {
        width: EXTENT,
        height: EXTENT,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world");
    world
        .set_choice_schedule(0)
        .expect("the exponent is inside the range");
    world
}

/// Returns the number of tiles of one cell that admit a unit.
fn open_tiles_of(world: &World, cell: u32) -> i64 {
    world
        .pyramid()
        .cell(cell)
        .expect("the cell is inside the level")
        .open_tiles()
}

/// Returns the cell that lies in one direction from another.
fn neighbour_cell(world: &World, cell: u32, direction: usize) -> Option<u32> {
    let cells = world.exit_field().cells();
    let here = cells.address_of(TileIdx(cell))?;
    let there = cells.neighbour(here, direction)?;
    Some(cells.index_of(there)?.0)
}

#[test]
fn no_cell_points_at_ground_that_admits_nobody() {
    // The exit field ranks its neighbours on a summary field, and no summary
    // field says whether a unit may stand in the cell. A cell of open water
    // can therefore outrank dry ground, and a whole block is then sent at a
    // coast it can never cross.[^1]
    //
    // [^1]: Findings register, FND-315. `docs/FINDINGS.md`
    let mut world = world(CLOSED_NEIGHBOUR_SEED);
    world.rebuild_pyramid(1).expect("the rebuild must run");

    // **The fixture must hold a cell that would name closed ground.** Ground
    // that admits nobody is common, and a test that only asserted that much
    // would pass on a world where no cell ever ranked such a neighbour first,
    // which is most worlds. The count below is the case itself: a neighbour
    // that admits nobody and that beats the ground the unit stands on.[^1]
    //
    // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
    let cells = world.exit_field().cells().tile_count();
    assert!(
        closed_neighbours_that_win(&world) > 0,
        "the fixture holds no cell that ranks closed ground above itself, \
         so the assertion below cannot fail"
    );

    let field = world.exit_field();
    for cell in 0..cells {
        for option in 0..choose::OPTION_COUNT as u8 {
            let Some(Some(direction)) = field.exit(cell, option) else {
                continue;
            };
            let there = neighbour_cell(&world, cell, direction as usize)
                .expect("the exit names a neighbour inside the lattice");
            assert_ne!(
                open_tiles_of(&world, there),
                0,
                "cell {cell} option {option} points at cell {there}, \
                 which admits nobody"
            );
        }
    }
}

#[test]
fn a_unit_the_ground_refuses_leaves_the_tile_it_started_on() {
    // A unit whose cell exit names water reads the same cell, the same option
    // and the same direction on every frame, so the refusal repeats and the
    // unit never moves. The fall-back draw is keyed on the frame, so the unit
    // takes a different direction on the next frame.[^1]
    //
    // [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    let mut world = world(REFUSED_SEED);
    world.rebuild_pyramid(1).expect("the rebuild must run");

    let start = a_tile_whose_exit_is_refused(&world);
    let unit = world
        .spawn_soldier(start, FactionId(0))
        .expect("the ground admits the unit");

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
        "the unit held tile {start:?} for {FRAMES} frames, so a refused \
         direction still freezes it"
    );
}

/// Returns a tile whose cell holds an exit that the ground under that tile
/// refuses.
///
/// The scan runs in ascending tile order and takes the first tile that fits,
/// so the answer is fixed and does not depend on how a caller walked the
/// world.[^1]
///
/// The tile is the whole fixture. A test that placed a unit on a tile chosen
/// for how it looks would assert against a case that the engine never
/// reaches.[^2]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
/// [^2]: Testing rules, section 2a. `.claude/rules/testing.md`
fn a_tile_whose_exit_is_refused(world: &World) -> Axial {
    let grid = world.grid();
    let layout = world.pyramid().layout();
    let field = world.exit_field();
    for tile in 0..grid.tile_count() {
        let here = grid
            .address_of(TileIdx(tile))
            .expect("the tile is inside the world");
        if !world.admits_a_unit(here) {
            continue;
        }
        let cell = layout.block_of_key(
            layout
                .key_of(TileIdx(tile))
                .expect("the tile is inside the world"),
        );
        for option in 0..choose::OPTION_COUNT as u8 {
            let Some(Some(direction)) = field.exit(cell, option) else {
                continue;
            };
            let refused = grid
                .neighbour(here, direction as usize)
                .is_none_or(|target| !world.admits_a_unit(target));
            if refused {
                return here;
            }
        }
    }
    panic!("the fixture holds no tile whose cell exit the ground refuses");
}

/// Returns the number of times a cell ranks a closed neighbour above itself.
///
/// The count reads the ranking rule rather than the field, so it answers what
/// the field would hold without the refusal. A fixture check that read the
/// field itself would read the answer the fix already wrote, and it would
/// pass on every world.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn closed_neighbours_that_win(world: &World) -> u32 {
    let cells = world.exit_field().cells();
    let mut found = 0;
    for cell in 0..cells.tile_count() {
        let Some(mine) = world.pyramid().cell(cell) else {
            continue;
        };
        if mine.open_tiles() == 0 {
            continue;
        }
        let Some(here) = cells.address_of(TileIdx(cell)) else {
            continue;
        };
        for row in &choose::OPTIONS {
            let mut best = choose::field_value(mine, row.field);
            for direction in 0..NEIGHBOURS.len() {
                let Some(there) = cells.neighbour(here, direction) else {
                    continue;
                };
                let Some(index) = cells.index_of(there) else {
                    continue;
                };
                let Some(summary) = world.pyramid().cell(index.0) else {
                    continue;
                };
                let value = choose::field_value(summary, row.field);
                if value > best {
                    best = value;
                    if summary.open_tiles() == 0 {
                        found += 1;
                    }
                }
            }
        }
    }
    found
}
