//! The proof that the sector tests of the ring frame can fail.
//!
//! A test that compares a run against itself always passes and proves
//! nothing. Each test here takes the sector rule of the frame, perturbs it by
//! one, and asserts that the boundary assertion of the frame then rejects
//! it.[^1]
//!
//! The perturbations are the two off-by-one shapes that the sector arithmetic
//! can take. The first rotates every sector by one, which is what a wrong
//! axis order gives. The second swaps the two halves of every sextant, which
//! is what a reversed half test gives. Both are invisible in an aggregate,
//! because every tile still lands in some cell and every total still adds up.
//!
//! # References
//!
//! [^1]: Testing Rules, section 1. `.agents/rules/testing.md`

use cachette_core::hex::Axial;
use cachette_core::obs_ring::{ring_position, sextant_of, twelfth_of};

/// Returns the six hex direction vectors, in the order the sectors follow.
fn axes() -> Vec<Axial> {
    (0..6).map(|wedge| ring_position(1, wedge)).collect()
}

/// Returns the sector of a delta, rotated by one sector.
///
/// This is the answer a frame gives when its axis table starts on the wrong
/// direction.
fn rotated(delta: Axial) -> Option<u32> {
    twelfth_of(delta).map(|sector| (sector + 1) % 12)
}

/// Returns the sector of a delta, with the two halves of the sextant swapped.
///
/// This is the answer a frame gives when the half test compares the wrong
/// way round.
fn halves_swapped(delta: Axial) -> Option<u32> {
    twelfth_of(delta).map(|sector| sector ^ 1)
}

/// Returns the sextant of a delta, rotated by one sextant.
fn sextant_rotated(delta: Axial) -> Option<u32> {
    sextant_of(delta).map(|wedge| (wedge + 1) % 6)
}

#[test]
fn the_axis_assertion_rejects_a_rotated_sector() {
    let mut rejected = 0u32;
    for (wedge, axis) in axes().iter().enumerate() {
        if rotated(*axis) != Some(wedge as u32 * 2) {
            rejected += 1;
        }
    }
    assert_eq!(rejected, 6, "the axis assertion admitted a rotated sector");
}

#[test]
fn the_axis_assertion_rejects_a_rotated_sextant() {
    let mut rejected = 0u32;
    for (wedge, axis) in axes().iter().enumerate() {
        if sextant_rotated(*axis) != Some(wedge as u32) {
            rejected += 1;
        }
    }
    assert_eq!(rejected, 6, "the axis assertion admitted a rotated sextant");
}

#[test]
fn the_bisector_assertion_rejects_swapped_halves() {
    let axes = axes();
    let mut rejected = 0u32;
    for wedge in 0..6usize {
        let bisector = axes[wedge].add(axes[(wedge + 1) % 6]);
        if halves_swapped(bisector) != Some(wedge as u32 * 2 + 1) {
            rejected += 1;
        }
    }
    assert_eq!(
        rejected, 6,
        "the bisector assertion admitted swapped halves"
    );
}

#[test]
fn the_boundary_assertion_rejects_swapped_halves_on_every_axis() {
    let mut rejected = 0u32;
    for (wedge, axis) in axes().iter().enumerate() {
        let on = Axial::new(axis.q * 8, axis.r * 8);
        if halves_swapped(on) != Some(wedge as u32 * 2) {
            rejected += 1;
        }
    }
    assert_eq!(
        rejected, 6,
        "the boundary assertion admitted swapped halves"
    );
}

#[test]
fn the_equal_count_assertion_survives_a_rotation_and_the_boundary_test_does_not() {
    let mut rotated_counts = [0u32; 12];
    for step in 0..6 * 20u32 {
        let delta = ring_position(20, step);
        let sector = rotated(delta).expect("a ring position has a sector");
        rotated_counts[sector as usize] += 1;
    }
    for count in rotated_counts {
        assert_eq!(
            count, 10,
            "a count over a whole ring cannot see a rotation, which is why the boundary test exists"
        );
    }
    let axis = ring_position(1, 0);
    assert_ne!(rotated(axis), Some(0));
}
