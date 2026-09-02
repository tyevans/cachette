//! The influence field.
//!
//! The tests go through the public crate API. They do not reach into an
//! internal module.[^1]
//!
//! **The fixture is built for the extremes, not for the typical case.** The
//! lattice holds open ground, ground that resists influence, and ground that
//! stops it. It holds a source at the centre of the open half and a source on
//! the edge of the lattice. A fixture that modelled a plausible world would
//! supply no extreme, and the assertions would then measure the fixture.[^2]
//!
//! # References
//!
//! [^1]: Testing policy. `docs/TESTING.md`
//! [^2]: Testing rules, section 2a. `.claude/rules/testing.md`

use cachette_core::influence::PASSES_FOR_EACH_SOLVE;
use cachette_core::{Axial, Conductance, FactionId, Grid, Influence, InfluenceField, InfluenceError};
use cachette_core::{World, WorldConfig};
use proptest::prelude::*;

/// The edge of the cell lattice that the fixture builds.
///
/// It is wider than the reach of one source, so a cell beyond the reach is
/// part of the fixture rather than an accident of the size.
const EDGE: u32 = 24;

/// The column that the resistant ground stands in.
const WALL_COLUMN: i32 = 12;

/// The row that the resistant ground stops at. It does not span the lattice,
/// so an equally distant cell on open ground exists.
const WALL_END: i32 = 16;

/// How freely the resistant ground carries influence.
///
/// It is low and it is not zero. Ground that carried nothing would make the
/// far side exactly zero, and an assertion against zero cannot tell an
/// obstruction from a field that never arrived.
const WALL: u8 = 128;

/// The faction whose source sits at the centre of the open half.
const SEATED: FactionId = FactionId(0);

/// The faction whose source sits on the edge of the lattice.
const EDGED: FactionId = FactionId(1);

/// The cell that the seated faction injects at.
const SEAT: Axial = Axial::new(6, 8);

/// The cell that the edged faction injects at.
const EDGE_SEAT: Axial = Axial::new(0, 12);

/// The distance that the two comparison cells sit at.
const REACH: i32 = 8;

/// A cell at the comparison distance, reached across the resistant ground.
const SHELTERED: Axial = Axial::new(SEAT.q + REACH, SEAT.r);

/// A cell at the comparison distance, reached over open ground.
const EXPOSED: Axial = Axial::new(SEAT.q, SEAT.r + REACH);

/// A cell of ground that stops influence. Nothing injects at it.
const STOPPED: Axial = Axial::new(2, 20);

/// The solves that bring the fixture to rest.
///
/// The field stops changing well before this. The count is generous rather
/// than tuned, because a test that depended on the exact count would be a
/// test of the count.
const SOLVES_TO_REST: usize = 30;

/// Builds the fixture lattice.
fn lattice() -> Grid {
    Grid::new(EDGE, EDGE).expect("the extent must describe a grid")
}

/// Builds the fixture: two factions, a wall, a patch of stopped ground, and
/// the two sources.
fn fixture() -> InfluenceField {
    let cells = lattice();
    let mut field = InfluenceField::new(cells, 2).expect("two factions are inside the ceiling");
    field
        .set_conductance(conductance())
        .expect("the plane covers the lattice");
    assert!(field.set_source(SEATED, SEAT, Influence::UNIT));
    assert!(field.set_source(EDGED, EDGE_SEAT, Influence::UNIT));
    field
}

/// Builds the conductance plane of the fixture.
fn conductance() -> Vec<Conductance> {
    let cells = lattice();
    let mut plane = vec![Conductance::FREE; cells.tile_count() as usize];
    let mut set = |address: Axial, value: Conductance| {
        let index = cells
            .index_of(address)
            .expect("the address is inside the lattice");
        plane[index.0 as usize] = value;
    };
    for row in 0..WALL_END {
        set(Axial::new(WALL_COLUMN, row), Conductance(WALL));
    }
    for column in 2..4 {
        for row in 20..22 {
            set(Axial::new(column, row), Conductance::BLOCKED);
        }
    }
    plane
}

/// Runs a field to rest.
fn to_rest(field: &mut InfluenceField, threads: usize) {
    for _ in 0..SOLVES_TO_REST {
        field.solve(threads).expect("the thread count is not zero");
    }
}

/// Returns what a faction holds at a cell.
fn at(field: &InfluenceField, faction: FactionId, cell: Axial) -> u16 {
    field
        .at(faction, cell)
        .expect("the faction and the cell are inside the field")
        .0
}

#[test]
fn a_source_raises_the_cell_it_sits_on_and_the_cells_around_it() {
    let mut field = fixture();
    to_rest(&mut field, 1);

    assert_eq!(
        at(&field, SEATED, SEAT),
        Influence::UNIT.0,
        "the source holds the cell it sits on at the ceiling"
    );

    // The falloff is read along open ground, away from the resistant ground
    // and away from the edge of the lattice.
    let mut previous = Influence::UNIT.0;
    let mut reached = 0;
    for step in 1..=REACH {
        let value = at(&field, SEATED, Axial::new(SEAT.q, SEAT.r + step));
        assert!(
            value < previous,
            "the field at {step} cells is {value}, which is not below the {previous} \
             at {} cells",
            step - 1
        );
        previous = value;
        if value > 0 {
            reached = step;
        }
    }
    assert_eq!(
        reached, REACH,
        "the source reached only {reached} cells, so the falloff test measures \
         the reach and not the falloff"
    );
}

#[test]
fn each_faction_holds_its_own_plane() {
    let mut field = fixture();
    to_rest(&mut field, 1);

    // Each plane holds the ceiling at the cell its own source sits on, and
    // nowhere else. The two sources are near enough to reach each other, so a
    // plane that read the wrong source would still hold a value here, and
    // only the ceiling tells them apart.
    let cells = lattice();
    for faction in [SEATED, EDGED] {
        let seat = if faction == SEATED { SEAT } else { EDGE_SEAT };
        let mut ceilings = Vec::new();
        for index in 0..cells.tile_count() {
            let address = cells
                .address_of(cachette_core::TileIdx(index))
                .expect("the index is inside the lattice");
            if at(&field, faction, address) == Influence::UNIT.0 {
                ceilings.push(address);
            }
        }
        assert_eq!(ceilings, vec![seat]);
    }

    assert!(at(&field, EDGED, SEAT) < at(&field, SEATED, SEAT));
    assert!(at(&field, SEATED, EDGE_SEAT) < at(&field, EDGED, EDGE_SEAT));
}

#[test]
fn ground_that_resists_influence_obstructs_it() {
    let mut field = fixture();
    to_rest(&mut field, 1);

    assert_eq!(
        SEAT.distance(SHELTERED),
        SEAT.distance(EXPOSED),
        "the two comparison cells must sit at one distance"
    );

    let sheltered = at(&field, SEATED, SHELTERED);
    let exposed = at(&field, SEATED, EXPOSED);
    assert!(
        exposed > 0,
        "the source did not reach the open cell, so the comparison measures \
         the reach and not the ground"
    );
    assert!(
        sheltered < exposed,
        "the cell behind the resistant ground holds {sheltered} and the cell \
         at the same distance over open ground holds {exposed}"
    );
}

#[test]
fn ground_that_stops_influence_holds_nothing() {
    let mut field = fixture();
    to_rest(&mut field, 1);
    assert_eq!(
        at(&field, SEATED, STOPPED),
        0,
        "influence entered a cell that conducts nothing"
    );
}

#[test]
fn the_solve_runs_the_fixed_pass_count_whatever_the_input() {
    // A field that holds nothing, a field at rest, a field that saturates,
    // and a field whose ground stops everything. Each of them is a case a
    // convergence test would cut short, and the pass count is the same.
    let empty = InfluenceField::new(lattice(), 2).expect("two factions are inside the ceiling");

    let mut saturated =
        InfluenceField::new(lattice(), 1).expect("one faction is inside the ceiling");
    for row in 0..EDGE as i32 {
        for column in 0..EDGE as i32 {
            assert!(saturated.set_source(SEATED, Axial::new(column, row), Influence::UNIT));
        }
    }

    let mut stopped = InfluenceField::new(lattice(), 1).expect("one faction is inside the ceiling");
    stopped
        .set_conductance(vec![
            Conductance::BLOCKED;
            lattice().tile_count() as usize
        ])
        .expect("the plane covers the lattice");

    for mut field in [empty, fixture(), saturated, stopped] {
        for solves in 1..=4u64 {
            field.solve(1).expect("the thread count is not zero");
            assert_eq!(
                field.passes(),
                solves * u64::from(PASSES_FOR_EACH_SOLVE),
                "a solve ran a pass count that the input decided"
            );
        }
        // A field already at rest runs the same passes as a field that is
        // moving. This is the case a convergence test cuts short first.
        let before = field.passes();
        for _ in 0..SOLVES_TO_REST {
            field.solve(1).expect("the thread count is not zero");
        }
        assert_eq!(
            field.passes() - before,
            SOLVES_TO_REST as u64 * u64::from(PASSES_FOR_EACH_SOLVE)
        );
    }
}

#[test]
fn a_field_with_no_source_falls_from_the_edge_inward() {
    let mut field = fixture();
    to_rest(&mut field, 1);
    let far = EXPOSED;
    assert!(at(&field, SEATED, far) > 0);

    // The source goes. Nothing else changes, and no pass asks whether a
    // source is there.
    assert!(field.set_source(SEATED, SEAT, Influence::ZERO));

    let mut fell = None;
    for round in 1..=SOLVES_TO_REST {
        field.solve(1).expect("the thread count is not zero");
        if at(&field, SEATED, far) == 0 {
            fell = Some(round);
            break;
        }
    }
    let round = fell.expect("the far cell never fell, so the fixture never lost its source");
    assert!(
        at(&field, SEATED, SEAT) > 0,
        "the seat lost its hold in the same solve as the far cell, so the \
         test cannot tell the edge from the centre"
    );
    assert!(round > 1, "the whole field fell in one solve");
}

#[test]
fn a_solve_at_no_threads_is_refused() {
    let mut field = fixture();
    assert_eq!(field.solve(0), Err(InfluenceError::ZeroThreads));
    assert_eq!(field.passes(), 0);
}

/// Returns every cell of every plane, after a run at one thread count.
fn every_cell(threads: usize, sources: &[(FactionId, Axial)]) -> Vec<u16> {
    let mut field = fixture();
    for (faction, cell) in sources {
        assert!(field.set_source(*faction, *cell, Influence::UNIT));
    }
    to_rest(&mut field, threads);
    let cells = lattice();
    let mut out = Vec::new();
    for faction in 0..field.faction_count() {
        for index in 0..cells.tile_count() {
            let address = cells
                .address_of(cachette_core::TileIdx(index))
                .expect("the index is inside the lattice");
            out.push(at(&field, FactionId(faction), address));
        }
    }
    out
}

#[test]
fn the_field_is_identical_at_every_thread_count() {
    let at_one = every_cell(1, &[]);
    assert!(
        at_one.iter().any(|value| *value > 0),
        "the fixture produced an empty field, so the comparison proves nothing"
    );
    assert_eq!(at_one, every_cell(2, &[]));
    assert_eq!(at_one, every_cell(12, &[]));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// The field is identical, cell for cell, at one, two and twelve threads,
    /// wherever the sources sit.
    #[test]
    fn the_field_is_identical_at_every_thread_count_wherever_the_sources_sit(
        columns in prop::collection::vec(0..EDGE as i32, 3),
        rows in prop::collection::vec(0..EDGE as i32, 3),
    ) {
        let sources: Vec<(FactionId, Axial)> = columns
            .iter()
            .zip(&rows)
            .enumerate()
            .map(|(ordinal, (column, row))| {
                (FactionId((ordinal % 2) as u16), Axial::new(*column, *row))
            })
            .collect();
        let at_one = every_cell(1, &sources);
        prop_assert_eq!(&at_one, &every_cell(2, &sources));
        prop_assert_eq!(&at_one, &every_cell(12, &sources));
    }
}

/// The extent of the world that the engine tests read.
///
/// It covers more than one level 1 cell, so a source in one cell reaches a
/// neighbouring cell and the read is not the write.
const WORLD_EDGE: u32 = 256;

/// Builds a world that holds two factions.
fn world() -> World {
    World::new(WorldConfig {
        width: WORLD_EDGE,
        height: WORLD_EDGE,
        seed: 0x0cac_4e77_0104,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent must describe a world")
}

#[test]
fn the_step_solves_the_field_and_a_watcher_reads_a_cell() {
    // The engine is obligated to run the solve, so the test drives the engine
    // and then reads the field. A test that called the solve itself would
    // prove that the solver works and not that anything reaches it.[^1]
    //
    // [^1]: Testing rules, section 5. `.claude/rules/testing.md`
    let mut world = world();
    let seat = Axial::new(48, 48);
    let neighbour = Axial::new(48 + 32, 48);
    assert!(world.set_influence_source(FactionId(0), seat, Influence::UNIT));

    assert_eq!(world.influence(FactionId(0), seat), Some(Influence::ZERO));
    let before = world.influence_field().passes();
    for _ in 0..8 {
        world.step(2).expect("the step must run");
    }
    assert_eq!(
        world.influence_field().passes() - before,
        8 * u64::from(PASSES_FOR_EACH_SOLVE),
        "the step ran a pass count that the world decided"
    );

    assert_eq!(world.influence(FactionId(0), seat), Some(Influence::UNIT));
    let reached = world
        .influence(FactionId(0), neighbour)
        .expect("the address is inside the world");
    assert!(
        reached > Influence::ZERO,
        "the source did not reach the neighbouring cell"
    );
    assert!(reached < Influence::UNIT, "the field did not fall");
    assert_eq!(
        world.influence(FactionId(1), seat),
        Some(Influence::ZERO),
        "the source of one faction reached the plane of another"
    );
}

#[test]
fn a_world_refuses_a_source_it_does_not_hold() {
    let mut world = world();
    assert!(!world.set_influence_source(FactionId(9), Axial::new(0, 0), Influence::UNIT));
    assert!(!world.set_influence_source(
        FactionId(0),
        Axial::new(-1, 0),
        Influence::UNIT
    ));
    assert_eq!(world.influence(FactionId(9), Axial::new(0, 0)), None);
    assert_eq!(world.influence(FactionId(0), Axial::new(0, -1)), None);
}

#[test]
fn the_field_of_a_world_reads_the_ground_it_stands_on() {
    // The conductance of a cell follows the ground it covers, and the world
    // holds water. A world whose cells all conducted freely would leave the
    // ground rule untested.
    let world = world();
    let field = world.influence_field();
    let cells = field.cells();
    let mut lowest = Conductance::FREE;
    let mut highest = Conductance::BLOCKED;
    for index in 0..cells.tile_count() {
        let address = cells
            .address_of(cachette_core::TileIdx(index))
            .expect("the index is inside the lattice");
        let value = field
            .conductance(address)
            .expect("the address is inside the lattice");
        lowest = lowest.min(value);
        highest = highest.max(value);
    }
    assert!(
        lowest < highest,
        "every cell of the world conducts alike, so the ground rule is untested"
    );
}
