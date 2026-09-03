//! Property tests for the world geometry.
//!
//! The world is a rhombus, so a tile address is a raw axial pair and the
//! index is the row multiplied by the row length, plus the column.[^1] The
//! properties below are the laws that make that index usable: the address
//! and the index round-trip, the neighbour relation is symmetric, and the
//! edge does not wrap.[^2]
//!
//! The test sees only the public crate API.[^3]
//!
//! # References
//!
//! [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^2]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D2. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
//! [^3]: Testing policy. `docs/TESTING.md`

use cachette_core::hex::{GridError, NEIGHBOURS, NEIGHBOUR_COUNT};
use cachette_core::types::TileIdx;
use cachette_core::{Axial, Grid, World, WorldConfig};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// A strategy that produces a grid with a modest extent.
///
/// The extent stays small so that a property can walk every tile of the
/// world without the test becoming a benchmark.
fn any_grid() -> impl Strategy<Value = Grid> {
    (1u32..40, 1u32..40).prop_map(|(width, height)| {
        Grid::new(width, height).expect("a small extent must describe a grid")
    })
}

/// A strategy that produces a grid and an address inside it.
fn any_grid_and_address() -> impl Strategy<Value = (Grid, Axial)> {
    any_grid().prop_flat_map(|grid| {
        (0..grid.width() as i32, 0..grid.height() as i32)
            .prop_map(move |(q, r)| (grid, Axial::new(q, r)))
    })
}

/// A strategy that produces a grid of any legal width, and an index in it.
///
/// The width reaches the whole legal range rather than the modest extent the
/// strategy above uses. **The conversion from an index to an address
/// multiplies by a reciprocal that the width decides, so a narrow spread of
/// widths would exercise one reciprocal and report that the whole range
/// works.**[^1] The height is chosen after the width, so that the tile count
/// stays inside the ceiling that makes the reciprocal exact.
///
/// # References
///
/// [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
fn any_wide_grid_and_index() -> impl Strategy<Value = (Grid, u32)> {
    (1u32..=u32::MAX)
        .prop_flat_map(|width| {
            let tallest = (u32::MAX / width).max(1);
            (Just(width), 1u32..=tallest)
        })
        .prop_map(|(width, height)| {
            Grid::new(width, height).expect("the height was chosen to fit the ceiling")
        })
        .prop_flat_map(|grid| {
            let count = grid.tile_count();
            let width = grid.width();
            let rows = grid.height();
            // **The index is drawn at a row boundary, not uniformly.** A
            // reciprocal that is one too small gives the wrong quotient
            // exactly at a multiple of the width, and nowhere else. A uniform
            // index below the tile count lands on a multiple with probability
            // one over the width, so at a wide extent it never lands on one,
            // and the property passes against a broken reciprocal.
            //
            // This was not reasoned out in advance. The defect was put back
            // and the uniform version of this strategy stayed green.[^1]
            //
            // [^1]: Testing rules, section 2a. `.claude/rules/testing.md`
            (Just(grid), 0u32..rows, -1i64..=1).prop_map(move |(grid, row, offset)| {
                let at = i64::from(row) * i64::from(width) + offset;
                (grid, at.clamp(0, i64::from(count) - 1) as u32)
            })
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        // An integration test has no lib.rs or main.rs above it, so the
        // default source-parallel persistence finds no root and silently
        // disables itself. A failing seed is then never written and never
        // replayed. Name the file, so that a seed which caught a defect runs
        // first on every later run.[^1]
        //
        // [^1]: Findings register, FND-044. `docs/FINDINGS.md`
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/hex_geometry.proptest-regressions"),
        ))),
        ..ProptestConfig::default()
    })]
    /// An address converts to an index and back to the same address.
    ///
    /// This is the property that makes the index an address rather than an
    /// arbitrary label.
    #[test]
    fn an_address_and_an_index_round_trip((grid, address) in any_grid_and_address()) {
        let index = grid.index_of(address).expect("an inside address has an index");
        let back = grid.address_of(index).expect("a valid index has an address");
        prop_assert_eq!(address, back);
    }

    /// Every index below the tile count converts to an address and back.
    ///
    /// The reverse direction of the property above. The viewer walks indices,
    /// so the round trip must hold from that side too.
    #[test]
    fn an_index_and_an_address_round_trip(grid in any_grid(), raw in 0u32..1_600) {
        prop_assume!(raw < grid.tile_count());
        let index = TileIdx(raw);
        let address = grid.address_of(index).expect("an inside index has an address");
        prop_assert_eq!(grid.index_of(address), Some(index));
    }

    /// The address of an index is the quotient and the remainder by the width.
    ///
    /// **This states the division that `address_of` no longer runs.** The
    /// conversion multiplies by a reciprocal that the grid computes once, and
    /// the claim that makes it legal is that the multiply gives the answer the
    /// division gives, for every index of every world the crate can
    /// build.[^2] A round trip through `index_of` cannot see a defect here,
    /// because `index_of` would undo a wrong split of the same index.
    ///
    /// [^2]: Findings register, FND-282. `docs/FINDINGS.md`
    #[test]
    fn an_address_is_the_quotient_and_the_remainder((grid, raw) in any_wide_grid_and_index()) {
        let address = grid.address_of(TileIdx(raw)).expect("an inside index has an address");
        prop_assert_eq!(address.q as u32, raw % grid.width());
        prop_assert_eq!(address.r as u32, raw / grid.width());
    }

    /// The index of an address is inside the world.
    #[test]
    fn an_index_is_inside_the_tile_count((grid, address) in any_grid_and_address()) {
        let index = grid.index_of(address).expect("an inside address has an index");
        prop_assert!(index.0 < grid.tile_count());
    }

    /// The neighbour relation is symmetric.
    ///
    /// If B is a neighbour of A, then A is a neighbour of B. A relation that
    /// is not symmetric makes movement one-way, and no test of a single
    /// direction would find it.
    #[test]
    fn the_neighbour_relation_is_symmetric((grid, address) in any_grid_and_address()) {
        for direction in 0..NEIGHBOUR_COUNT {
            let Some(neighbour) = grid.neighbour(address, direction) else {
                continue;
            };
            let back = grid.neighbours(neighbour);
            prop_assert!(
                back.contains(&Some(address)),
                "{address:?} is a neighbour of {neighbour:?} in no direction",
            );
        }
    }

    /// A tile has six neighbour slots, whatever its position.
    ///
    /// The slot count is fixed so that a caller reads a direction by its
    /// index. An edge tile has absent neighbours, not fewer slots.
    #[test]
    fn every_tile_has_six_neighbour_slots((grid, address) in any_grid_and_address()) {
        prop_assert_eq!(grid.neighbours(address).len(), NEIGHBOUR_COUNT);
    }

    /// Every neighbour that exists is one step away.
    #[test]
    fn a_neighbour_is_one_step_away((grid, address) in any_grid_and_address()) {
        for neighbour in grid.neighbours(address).into_iter().flatten() {
            prop_assert_eq!(address.distance(neighbour), 1);
        }
    }

    /// No tile is its own neighbour.
    #[test]
    fn a_tile_is_not_its_own_neighbour((grid, address) in any_grid_and_address()) {
        for neighbour in grid.neighbours(address).into_iter().flatten() {
            prop_assert_ne!(neighbour, address);
        }
    }

    /// Distance is symmetric, and zero only between an address and itself.
    #[test]
    fn distance_is_symmetric_and_zero_only_at_home(
        (grid, first) in any_grid_and_address(),
        (_, second) in any_grid_and_address(),
    ) {
        let _ = grid;
        prop_assert_eq!(first.distance(second), second.distance(first));
        prop_assert_eq!(first.distance(first), 0);
        if first != second {
            prop_assert!(first.distance(second) > 0);
        }
    }

    /// An address outside the world has no index, in any direction.
    #[test]
    fn an_outside_address_has_no_index(grid in any_grid(), q in -8i32..48, r in -8i32..48) {
        let address = Axial::new(q, r);
        let inside = q >= 0
            && r >= 0
            && (q as u32) < grid.width()
            && (r as u32) < grid.height();
        prop_assert_eq!(grid.index_of(address).is_some(), inside);
        prop_assert_eq!(grid.contains(address), inside);
    }

    /// The world reads a tile value through an address, and refuses one
    /// outside the world.
    ///
    /// This drives the engine rather than the geometry, so it proves that
    /// the grid has a real caller.
    #[test]
    fn the_world_reads_a_tile_through_an_address(width in 1u32..24, height in 1u32..24) {
        let world = World::new(WorldConfig {
            width,
            height,
            ..WorldConfig::default()
        })
        .expect("a small extent must describe a world");

        let corner = Axial::new(width as i32 - 1, height as i32 - 1);
        prop_assert!(world.tile_value(corner).is_some());
        prop_assert!(world.tile_value(Axial::new(width as i32, 0)).is_none());
        prop_assert!(world.tile_value(Axial::new(0, height as i32)).is_none());
        prop_assert!(world.tile_value(Axial::new(-1, 0)).is_none());
        prop_assert_eq!(world.grid().tile_count() as usize, world.tile_count());
    }
}

/// The corner of a one-tile world has no neighbour at all.
///
/// The smallest world is the case where every direction leaves the world, so
/// it is the case a wrapping defect would show in first.
#[test]
fn a_single_tile_world_has_no_neighbour() {
    let grid = Grid::new(1, 1).expect("one tile is a world");
    let address = Axial::new(0, 0);
    assert_eq!(grid.tile_count(), 1);
    assert!(grid.neighbours(address).iter().all(Option::is_none));
}

/// A corner of a larger world has the neighbour count that its position
/// allows, and no more.
///
/// The counts are read from the neighbour offsets rather than asserted as
/// bare numbers, so the test states the reason and not only the answer.
#[test]
fn a_corner_has_only_the_neighbours_inside_the_world() {
    let grid = Grid::new(4, 4).expect("a small extent is a world");
    for (name, address) in [
        ("origin", Axial::new(0, 0)),
        ("far column", Axial::new(3, 0)),
        ("far row", Axial::new(0, 3)),
        ("far corner", Axial::new(3, 3)),
    ] {
        let expected = NEIGHBOURS
            .iter()
            .filter(|offset| grid.contains(address.add(**offset)))
            .count();
        let actual = grid
            .neighbours(address)
            .iter()
            .filter(|n| n.is_some())
            .count();
        assert_eq!(
            actual, expected,
            "corner {name} has the wrong neighbour count"
        );
        assert!(actual < NEIGHBOUR_COUNT, "corner {name} is not on an edge");
    }
}

/// An interior tile has all six neighbours.
#[test]
fn an_interior_tile_has_six_neighbours() {
    let grid = Grid::new(5, 5).expect("a small extent is a world");
    let neighbours = grid.neighbours(Axial::new(2, 2));
    assert_eq!(
        neighbours.iter().filter(|n| n.is_some()).count(),
        NEIGHBOUR_COUNT
    );
}

/// A direction outside the fixed set has no neighbour.
#[test]
fn an_unknown_direction_has_no_neighbour() {
    let grid = Grid::new(4, 4).expect("a small extent is a world");
    assert_eq!(grid.neighbour(Axial::new(1, 1), NEIGHBOUR_COUNT), None);
}

/// A world with an empty side is not a world.
#[test]
fn an_empty_side_is_refused() {
    assert_eq!(Grid::new(0, 4), Err(GridError::EmptySide));
    assert_eq!(Grid::new(4, 0), Err(GridError::EmptySide));
    assert!(World::new(WorldConfig {
        width: 0,
        ..WorldConfig::default()
    })
    .is_err());
}

/// An extent whose tile count overflows the index type is refused.
///
/// The index is a 32-bit value, so the refusal is what stops a silent wrap
/// rather than a bounds check on every access.
#[test]
fn an_extent_that_overflows_the_index_is_refused() {
    assert_eq!(Grid::new(u32::MAX, 2), Err(GridError::TooManyTiles));
    assert!(Grid::new(65_536, 65_536).is_err());
}

/// The neighbour offsets are the six distinct unit steps.
///
/// The order of the offsets is fixed, and a system that iterates them relies
/// on the order. This test pins the set rather than the order, and the
/// symmetry property above pins what the order must satisfy.
#[test]
fn the_neighbour_offsets_are_six_distinct_unit_steps() {
    let origin = Axial::new(0, 0);
    for offset in NEIGHBOURS {
        assert_eq!(origin.distance(offset), 1);
    }
    for (index, first) in NEIGHBOURS.iter().enumerate() {
        for second in &NEIGHBOURS[index + 1..] {
            assert_ne!(first, second);
        }
    }
}

/// Every index of every narrow world splits the way a division splits it.
///
/// The property above samples. This walks every index of every world up to a
/// small edge, so the narrow widths are covered exhaustively rather than by
/// chance. A width of one and a width of two are the two cases the reciprocal
/// treats specially or nearly so.
#[test]
fn a_narrow_world_splits_every_index_the_way_a_division_does() {
    for width in 1u32..=64 {
        for height in 1u32..=64 {
            let grid = Grid::new(width, height).expect("a small extent must describe a grid");
            for raw in 0..grid.tile_count() {
                let address = grid
                    .address_of(TileIdx(raw))
                    .expect("an inside index has an address");
                assert_eq!(
                    (address.q as u32, address.r as u32),
                    (raw % width, raw / width),
                    "width {width}, height {height}, index {raw}"
                );
            }
        }
    }
}

/// The widest and the narrowest worlds split their extreme indices correctly.
///
/// The reciprocal is exact because the tile count stays inside a `u32`. These
/// are the worlds that sit on that bound, so they are the ones an error in the
/// bound would reach first.
#[test]
fn the_extreme_worlds_split_their_extreme_indices() {
    let extremes = [
        (1u32, u32::MAX),
        (u32::MAX, 1u32),
        (2, u32::MAX / 2),
        (u32::MAX / 2, 2),
        (65_535, 65_535),
    ];
    for (width, height) in extremes {
        let grid = Grid::new(width, height).expect("the extent must describe a world");
        let count = grid.tile_count();
        // A world one row tall has no index at the width, so the probes are
        // filtered rather than assumed.
        let probes = [0, 1, width.saturating_sub(1), width, count / 2, count - 1];
        for raw in probes.into_iter().filter(|raw| *raw < count) {
            let address = grid
                .address_of(TileIdx(raw))
                .expect("an inside index has an address");
            assert_eq!(
                (address.q as u32, address.r as u32),
                (raw % width, raw / width),
                "width {width}, height {height}, index {raw}"
            );
        }
    }
}
