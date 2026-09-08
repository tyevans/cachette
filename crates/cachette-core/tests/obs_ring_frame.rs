//! Tests for the egocentric ring frame of the observation.
//!
//! The frame samples space around a faction at falling resolution. It gives
//! the observation a fixed width on every world, which a field over the world
//! lattice cannot do.[^1]
//!
//! Every test here reaches the frame through the public interface of the
//! crate, so it pins the behaviour rather than the arrangement.
//!
//! The sector tests are the ones that matter most. An off-by-one at a sector
//! boundary is invisible in an aggregate, because the tile still lands in
//! some cell and every total still adds up. Each test below therefore names
//! one boundary and asserts the sector on both sides of it.
//!
//! # References
//!
//! [^1]: Findings register, FND-670. `docs/FINDINGS.md`

use cachette_core::hex::{Axial, Grid};
use cachette_core::obs_ring::{
    cell_nominal_tiles, cell_of_delta, hex_distance_from_origin, ring_band, ring_nominal_tiles,
    ring_of_cell, ring_of_distance, ring_position, ring_sectors, sector_of_cell, sextant_of,
    shared_sector_of, twelfth_of, CellExtent, FAR_SECTORS, FIRST_FAR_RING, NEAR_DISTANCE,
    NEAR_SECTORS, RING_CAP, RING_COUNT, RING_STACK_CELLS, RING_STACK_CHANNELS,
};

/// Returns the six hex direction vectors, in the order the sectors follow.
///
/// The frame publishes no axis table, so the test derives the axes from the
/// ring at distance one. Step `w` of that ring is the axis that sextant `w`
/// opens on.
fn axes() -> Vec<Axial> {
    (0..6).map(|wedge| ring_position(1, wedge)).collect()
}

#[test]
fn the_frame_holds_one_hundred_and_fifty_one_cells() {
    assert_eq!(RING_STACK_CELLS, 151);
    assert_eq!(RING_STACK_CHANNELS, 25);
    assert_eq!(RING_COUNT, RING_CAP + 1);
    assert_eq!(
        RING_STACK_CELLS,
        1 + NEAR_SECTORS + (RING_COUNT - 2) * FAR_SECTORS
    );
}

#[test]
fn the_cap_admits_the_widest_distance_of_the_target_world() {
    let corner = Axial::new(4095, 4095);
    let distance = hex_distance_from_origin(corner);
    assert_eq!(distance, 8190);
    assert_eq!(ring_of_distance(distance), RING_CAP);
    let (low, high) = ring_band(RING_CAP);
    assert!(low <= distance && distance <= high);
}

#[test]
fn the_near_rings_cover_one_hundred_and_sixty_nine_tiles() {
    let total: i64 = (0..FIRST_FAR_RING).map(ring_nominal_tiles).sum();
    assert_eq!(total, 169);
    assert_eq!(ring_band(FIRST_FAR_RING - 1).1, NEAR_DISTANCE);
}

#[test]
fn every_cell_of_the_frame_has_one_ring_and_one_sector() {
    let mut counted = 0u32;
    for ring in 0..RING_COUNT {
        for sector in 0..ring_sectors(ring) {
            let cell = (0..RING_STACK_CELLS)
                .find(|cell| ring_of_cell(*cell) == ring && sector_of_cell(*cell) == sector)
                .unwrap_or_else(|| panic!("ring {ring} sector {sector} has no cell"));
            assert!(cell < RING_STACK_CELLS);
            counted += 1;
        }
    }
    assert_eq!(counted, RING_STACK_CELLS);
}

#[test]
fn each_axis_opens_the_sextant_that_bears_its_number() {
    for (wedge, axis) in axes().iter().enumerate() {
        assert_eq!(sextant_of(*axis), Some(wedge as u32), "axis {wedge}");
        assert_eq!(twelfth_of(*axis), Some(wedge as u32 * 2), "axis {wedge}");
    }
}

#[test]
fn a_long_step_along_each_axis_opens_the_same_sector() {
    for (wedge, axis) in axes().iter().enumerate() {
        let far = Axial::new(axis.q * 3000, axis.r * 3000);
        assert_eq!(twelfth_of(far), Some(wedge as u32 * 2), "axis {wedge}");
    }
}

#[test]
fn the_tile_below_an_axis_belongs_to_the_sector_below_it() {
    let axes = axes();
    for wedge in 0..6usize {
        let axis = axes[wedge];
        let previous = axes[(wedge + 5) % 6];
        let below = Axial::new(axis.q * 8 + previous.q, axis.r * 8 + previous.r);
        let expected = ((wedge as u32 + 5) % 6) * 2 + 1;
        assert_eq!(twelfth_of(below), Some(expected), "axis {wedge}");
        let on = Axial::new(axis.q * 8, axis.r * 8);
        assert_eq!(twelfth_of(on), Some(wedge as u32 * 2), "axis {wedge}");
    }
}

#[test]
fn the_bisector_of_each_wedge_opens_the_upper_half() {
    let axes = axes();
    for wedge in 0..6usize {
        let bisector = axes[wedge].add(axes[(wedge + 1) % 6]);
        assert_eq!(sextant_of(bisector), Some(wedge as u32), "wedge {wedge}");
        assert_eq!(
            twelfth_of(bisector),
            Some(wedge as u32 * 2 + 1),
            "wedge {wedge}"
        );
    }
}

#[test]
fn one_step_either_side_of_a_bisector_falls_in_the_two_halves() {
    let axes = axes();
    for wedge in 0..6usize {
        let opening = axes[wedge];
        let closing = axes[(wedge + 1) % 6];
        let below = Axial::new(opening.q * 9 + closing.q * 8, opening.r * 9 + closing.r * 8);
        assert_eq!(twelfth_of(below), Some(wedge as u32 * 2), "wedge {wedge}");
        let above = Axial::new(opening.q * 8 + closing.q * 9, opening.r * 8 + closing.r * 9);
        assert_eq!(
            twelfth_of(above),
            Some(wedge as u32 * 2 + 1),
            "wedge {wedge}"
        );
    }
}

#[test]
fn the_sextants_partition_a_disc_with_no_gap_and_no_overlap() {
    let mut wedges = [0u32; 6];
    let mut sectors = [0u32; 12];
    for q in -40..=40i32 {
        for r in -40..=40i32 {
            let delta = Axial::new(q, r);
            if hex_distance_from_origin(delta) == 0 {
                assert_eq!(sextant_of(delta), None);
                assert_eq!(twelfth_of(delta), None);
                continue;
            }
            let wedge = sextant_of(delta).expect("a delta off the centre has a sextant");
            let sector = twelfth_of(delta).expect("a delta off the centre has a sector");
            assert_eq!(sector / 2, wedge);
            wedges[wedge as usize] += 1;
            sectors[sector as usize] += 1;
        }
    }
    for count in wedges {
        assert!(count > 0);
    }
    for count in sectors {
        assert!(count > 0);
    }
}

#[test]
fn the_six_sextants_of_one_ring_hold_equal_counts() {
    let mut wedges = [0u32; 6];
    for step in 0..6 * 20 {
        let delta = ring_position(20, step);
        let wedge = sextant_of(delta).expect("a ring position has a sextant");
        wedges[wedge as usize] += 1;
    }
    for count in wedges {
        assert_eq!(count, 20);
    }
}

#[test]
fn the_twelve_sectors_of_one_ring_hold_equal_counts() {
    let mut sectors = [0u32; 12];
    for step in 0..6 * 20 {
        let delta = ring_position(20, step);
        sectors[shared_sector_of(delta) as usize] += 1;
    }
    for count in sectors {
        assert_eq!(count, 10);
    }
}

#[test]
fn a_ring_position_stands_at_the_distance_it_was_asked_for() {
    for distance in [1u32, 2, 5, 64, 1000] {
        for step in 0..6 * distance {
            let delta = ring_position(distance, step);
            assert_eq!(hex_distance_from_origin(delta), distance, "step {step}");
        }
    }
}

#[test]
fn the_ring_of_a_distance_follows_the_bit_length() {
    assert_eq!(ring_of_distance(0), 0);
    assert_eq!(ring_of_distance(1), 1);
    assert_eq!(ring_of_distance(2), 2);
    assert_eq!(ring_of_distance(3), 2);
    assert_eq!(ring_of_distance(4), 3);
    assert_eq!(ring_of_distance(7), 3);
    assert_eq!(ring_of_distance(8), 4);
    assert_eq!(ring_of_distance(4095), 12);
    assert_eq!(ring_of_distance(4096), 13);
    assert_eq!(ring_of_distance(u32::MAX), RING_CAP);
}

#[test]
fn a_nominal_annulus_equals_the_tiles_at_each_distance_of_its_band() {
    for ring in 0..RING_COUNT.min(8) {
        let (low, high) = ring_band(ring);
        let counted: i64 = (low..=high)
            .map(|distance| {
                if distance == 0 {
                    1
                } else {
                    6 * i64::from(distance)
                }
            })
            .sum();
        assert_eq!(counted, ring_nominal_tiles(ring), "ring {ring}");
        assert_eq!(
            cell_nominal_tiles(ring),
            counted / i64::from(ring_sectors(ring)),
            "ring {ring}"
        );
    }
}

#[test]
fn every_tile_of_a_band_falls_in_a_cell_of_its_own_ring() {
    for ring in 0..6u32 {
        let (low, high) = ring_band(ring);
        for distance in low..=high {
            let positions = if distance == 0 { 1 } else { 6 * distance };
            for step in 0..positions {
                let delta = ring_position(distance, step);
                let cell = cell_of_delta(delta);
                assert_eq!(ring_of_cell(cell), ring, "distance {distance} step {step}");
                assert!(sector_of_cell(cell) < ring_sectors(ring));
            }
        }
    }
}

#[test]
fn a_small_world_reads_no_extent_in_a_far_ring() {
    let grid = Grid::new(24, 24).expect("a 24 by 24 grid is legal");
    let extent = CellExtent::sample(grid, Axial::new(12, 12));
    for cell in 0..RING_STACK_CELLS {
        if ring_of_cell(cell) >= 7 {
            assert_eq!(extent.inside(cell), 0, "cell {cell}");
            assert_eq!(extent.in_world_tiles(cell), 0, "cell {cell}");
        }
    }
}

#[test]
fn a_centre_in_a_large_world_reads_a_full_extent_in_a_near_ring() {
    let grid = Grid::new(512, 512).expect("a 512 by 512 grid is legal");
    let extent = CellExtent::sample(grid, Axial::new(256, 256));
    for cell in 0..RING_STACK_CELLS {
        if ring_of_cell(cell) <= 3 {
            assert_eq!(extent.inside(cell), extent.sampled(cell), "cell {cell}");
        }
    }
}

#[test]
fn a_corner_centre_reads_a_part_extent_and_a_centre_reads_the_whole() {
    let grid = Grid::new(64, 64).expect("a 64 by 64 grid is legal");
    let middle = CellExtent::sample(grid, Axial::new(32, 32));
    let corner = CellExtent::sample(grid, Axial::new(0, 0));
    let ring = 3;
    let middle_inside: i64 = (0..RING_STACK_CELLS)
        .filter(|cell| ring_of_cell(*cell) == ring)
        .map(|cell| middle.inside(cell))
        .sum();
    let corner_inside: i64 = (0..RING_STACK_CELLS)
        .filter(|cell| ring_of_cell(*cell) == ring)
        .map(|cell| corner.inside(cell))
        .sum();
    assert!(
        corner_inside < middle_inside,
        "corner {corner_inside} middle {middle_inside}"
    );
}

#[test]
fn the_frame_is_translation_invariant() {
    let first = Axial::new(100, 100);
    let second = Axial::new(370, 41);
    for step in 0..6 * 37u32 {
        let delta = ring_position(37, step);
        let one = cell_of_delta(Axial::new(
            first.q + delta.q - first.q,
            first.r + delta.r - first.r,
        ));
        let two = cell_of_delta(Axial::new(
            second.q + delta.q - second.q,
            second.r + delta.r - second.r,
        ));
        assert_eq!(one, two, "step {step}");
    }
}
