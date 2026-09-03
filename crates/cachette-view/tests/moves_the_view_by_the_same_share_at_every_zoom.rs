//! One press of a scroll key covers the same part of the picture at every zoom.
//!
//! A press moved a fixed number of tiles. A tile is a fixed number of pixels
//! only at one zoom, so at the smallest tile the camera allows a press moved
//! three pixels and the camera felt stuck. Nothing was slow. The step was the
//! wrong size for the view.[^1]
//!
//! These tests assert the property the repair gives: **the pixel distance of a
//! press does not depend on the zoom.** One of them asserts the matching
//! property for the zoom itself, which multiplies rather than adds and
//! therefore never had the defect. That test exists so that a later change
//! from a ratio to a count fails rather than reintroducing the complaint in
//! the other direction.
//!
//! # References
//!
//! [^1]: Findings register, FND-209. `docs/FINDINGS.md`

// An integration test is its own crate, so the allowance at the viewer's
// crate root does not reach it. ADR-0067 D3 puts the float boundary at the
// viewer, and a camera is a viewer value.
#![allow(clippy::disallowed_types)]

use cachette_view::paint::{Camera, Canvas};

/// The window the tests steer in.
const WINDOW: (usize, usize) = (960, 720);

/// The zooms the tests sweep, from the smallest tile the camera allows to a
/// large one, with fractions between.
const WIDTHS: [f32; 8] = [2.0, 3.3, 6.0, 9.7, 12.0, 20.0, 33.0, 64.0];

/// Returns how far one press moves the view, in pixels.
fn press_distance(camera: Camera, canvas: &Canvas) -> (f32, f32) {
    let moved = camera.nudged(1.0, 1.0, canvas);
    (
        camera.origin_x - moved.origin_x,
        camera.origin_y - moved.origin_y,
    )
}

#[test]
fn one_press_moves_the_same_distance_at_every_zoom() {
    let canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let first = press_distance(Camera::at_tile_size(WIDTHS[0]), &canvas);
    for width in WIDTHS {
        let distance = press_distance(Camera::at_tile_size(width), &canvas);
        assert_eq!(
            distance, first,
            "a press moves {distance:?} pixels at a tile of {width} and {first:?} at the smallest",
        );
    }
}

#[test]
fn a_press_at_the_far_zoom_moves_a_readable_part_of_the_window() {
    // The complaint was that the camera would not go anywhere at the far zoom.
    // A press that moves under one part in two hundred of the window is a
    // press a person cannot see, whatever the arithmetic says.
    let canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let (across, _) = press_distance(Camera::at_tile_size(2.0), &canvas);
    assert!(
        across >= WINDOW.0 as f32 / 200.0,
        "a press at the far zoom moves {across} pixels in a window {} wide",
        WINDOW.0,
    );
}

#[test]
fn the_step_follows_the_window_and_not_the_world() {
    // The share is of the window, so a larger window pans further in one
    // press and crosses itself in the same number of presses.
    let small = Canvas::new(480, 360);
    let large = Canvas::new(960, 720);
    let camera = Camera::at_tile_size(12.0);
    let (small_step, _) = press_distance(camera, &small);
    let (large_step, _) = press_distance(camera, &large);
    assert!(
        (large_step - small_step * 2.0).abs() < 0.001,
        "a window of twice the side moves {large_step} against {small_step}",
    );
}

#[test]
fn the_zoom_at_the_opening_size_moves_what_it_moved_before() {
    // The share preserves the behaviour at the zoom the viewer opens on. That
    // step was one and a half tiles of twelve pixels. A repair that changed
    // every zoom, including the one nobody reported, would pass every
    // assertion above.
    let canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let camera = Camera::opening();
    let (across, down) = press_distance(camera, &canvas);
    assert!(
        (across - 18.0).abs() < 0.001 && (down - 18.0).abs() < 0.001,
        "the opening zoom moves {across} across and {down} down, and it moved 18 of each",
    );
}

#[test]
fn one_zoom_press_changes_the_view_by_a_constant_ratio() {
    // The zoom multiplies. A zoom that added a count of tiles would change the
    // view by a different proportion at each zoom, which is the same complaint
    // in the other direction.
    let canvas = Canvas::new(WINDOW.0, WINDOW.1);
    let mut ratios = Vec::new();
    for width in WIDTHS {
        let camera = Camera::at_tile_size(width);
        let closer = camera.zoomed_in(&canvas);
        // The camera clamps at its largest tile, so a zoom that is already
        // there has no ratio to report.
        if closer.tile_width < 64.0 {
            ratios.push(closer.tile_width / camera.tile_width);
        }
    }
    assert!(ratios.len() >= 4, "the sweep reached the clamp too early");
    let first = ratios[0];
    for ratio in &ratios {
        assert!(
            (ratio - first).abs() < 0.001,
            "one zoom press gives ratios of {ratios:?}, which are not one number",
        );
    }
}
