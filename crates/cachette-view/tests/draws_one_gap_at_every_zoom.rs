//! The window draws the same gap between two tiles at every zoom.
//!
//! A tile is a fractional number of pixels wide at nearly every zoom, because
//! each zoom step multiplies the size by a fraction. A drawing that placed a
//! fixed integer square at a rounded centre left a gap of one pixel under some
//! tiles and two pixels under others. The pattern repeated across the picture
//! and a watcher read it as a lattice.[^1]
//!
//! These tests assert the property that the repair gives: **one gap width
//! under every tile.** They read the rectangle from the drawing rather than
//! computing it again, and one of them reads the pixels the drawing wrote.[^2]
//!
//! The far zoom is the second half of the same defect. A tile two pixels
//! across that gives one pixel to a gap keeps a quarter of its own cell, so
//! three quarters of the picture is the gap. The drawing leaves the gap out
//! below the width at which the gap covers more of the cell than the tile.
//!
//! # References
//!
//! [^1]: Findings register, FND-207. `docs/FINDINGS.md`
//! [^2]: Testing Rules, section 6. `.claude/rules/testing.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a tile width in pixels is a viewer value.
#![allow(clippy::disallowed_types)]

use std::collections::BTreeSet;

use cachette_core::{Axial, World, WorldConfig};
use cachette_view::paint::{self, Camera, Canvas, BACKGROUND};

/// The window the tests draw into.
const WINDOW: (usize, usize) = (640, 400);

/// Returns a world with ground of several kinds.
fn world_of(seed: u64, extent: u32) -> World {
    World::new(WorldConfig {
        width: extent,
        height: extent,
        seed,
        faction_count: 2,
        unit_capacity: WorldConfig::TARGET_UNIT_POPULATION,
    })
    .expect("the extent describes a world")
}

/// The tile widths the tests sweep.
///
/// The list holds whole widths and fractional ones. A sweep of whole widths
/// only would pass against the drawing that had the defect, because the defect
/// needs a fraction to appear.
const WIDTHS: [f32; 12] = [2.0, 2.5, 3.0, 3.3, 4.0, 4.5, 5.0, 6.5, 8.0, 9.7, 12.0, 13.2];

/// Returns the gaps the drawing leaves along one row of tiles.
fn gaps_along_a_row(camera: Camera, row: i32, columns: std::ops::Range<i32>) -> BTreeSet<i32> {
    let mut gaps = BTreeSet::new();
    for column in columns {
        let (left, _, wide, _) = paint::tile_rect(camera, Axial::new(column, row));
        let (next, _, _, _) = paint::tile_rect(camera, Axial::new(column + 1, row));
        gaps.insert(next - left - wide);
    }
    gaps
}

#[test]
fn one_gap_width_stands_under_every_tile_at_every_zoom() {
    for width in WIDTHS {
        let camera = Camera::at_tile_size(width);
        let gaps = gaps_along_a_row(camera, 12, 0..120);
        assert_eq!(
            gaps.len(),
            1,
            "a tile {width} pixels wide leaves gaps of {gaps:?} across one row",
        );
    }
}

#[test]
fn the_same_gap_stands_between_two_rows() {
    // The rows are sheared by half a tile, so a repair that fixed the
    // horizontal gap alone would leave the lattice in the other direction.
    for width in WIDTHS {
        let camera = Camera::at_tile_size(width);
        let mut gaps = BTreeSet::new();
        for row in 0..120 {
            let (_, top, _, tall) = paint::tile_rect(camera, Axial::new(4, row));
            let (_, below, _, _) = paint::tile_rect(camera, Axial::new(4, row + 1));
            gaps.insert(below - top - tall);
        }
        assert_eq!(
            gaps.len(),
            1,
            "a tile {width} pixels tall leaves gaps of {gaps:?} down one column",
        );
    }
}

#[test]
fn a_tile_that_a_gap_would_swamp_is_drawn_with_no_gap() {
    // At the smallest tile the camera allows, a gap of one pixel would take
    // three quarters of the cell. The picture would then be mostly the colour
    // of the space outside the world.
    let camera = Camera::at_tile_size(2.0);
    let gaps = gaps_along_a_row(camera, 12, 0..120);
    assert_eq!(
        gaps,
        BTreeSet::from([0]),
        "the smallest tile the camera allows still gives room to a gap",
    );

    // The gap returns once the tile is wide enough to keep half of its cell.
    let camera = Camera::at_tile_size(8.0);
    let gaps = gaps_along_a_row(camera, 12, 0..120);
    assert_eq!(
        gaps,
        BTreeSet::from([1]),
        "a tile of eight pixels drew no separator",
    );
}

#[test]
fn the_grid_a_watcher_sees_holds_one_run_length() {
    // This test starts at the pixels. The two above read the arithmetic, and
    // arithmetic that agreed with itself and disagreed with the canvas would
    // pass both of them.
    let world = world_of(0x0cac_9a17, 96);
    let mut canvas = Canvas::new(WINDOW.0, WINDOW.1);
    // A fraction, because the defect needs one. The zoom step multiplies by a
    // fraction, so a person reaches a width like this by pressing the key.
    let camera = Camera::at_tile_size(6.5);
    paint::draw(&world, camera, &mut canvas).expect("the world draws");

    let row = WINDOW.1 / 2;
    let pixels = canvas.pixels();
    let mut runs = BTreeSet::new();
    let mut run = 0;
    // The sweep stays inside the world, away from the sheared ends of the row,
    // so a run does not reach the empty space beyond the last tile.
    for column in 120..520 {
        if pixels[row * WINDOW.0 + column] == BACKGROUND {
            run += 1;
        } else {
            if run > 0 {
                runs.insert(run);
            }
            run = 0;
        }
    }
    assert!(
        !runs.is_empty(),
        "the sweep found no gap, so it measures the fixture and not the drawing",
    );
    assert_eq!(
        runs.len(),
        1,
        "the grid a watcher sees holds runs of {runs:?} pixels, so it reads as a lattice",
    );
}
