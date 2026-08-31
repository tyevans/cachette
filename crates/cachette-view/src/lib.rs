//! The viewer for the Cachette simulation.
//!
//! This crate draws a world. It reads the world through the public interface
//! and writes nothing to it.[^1] The core crate does not depend on this one
//! and never will, which is what makes "the engine holds no value that
//! exists for the viewer" a compiler check rather than a reviewer's
//! judgement.[^2]
//!
//! Floating point begins here and does not return. Rendering is outside
//! simulated state, so the arithmetic is free, and no value that has been a
//! floating point number is handed back to the engine.[^3]
//!
//! The viewer runs after the step, on the stepping thread. The drawing rate
//! and the tick rate are therefore one number, and that is stated in the
//! record rather than left to be discovered.[^4]
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D5. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^3]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^4]: ADR-0067, the viewer reads the world and never writes to it, decision D4. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`

// The workspace bans the float types by name, because float addition is not
// associative and an aggregate must combine exactly.[^5] That ban protects
// simulated state. Rendering is outside simulated state, and the record that
// bans the types allows them here.[^3] The allowance is stated once, at the
// crate root, so a reader sees it before any code that uses it.
//
// The boundary is not held by this comment. It is held by the dependency
// direction and by the engine's types: the core does not depend on this
// crate, and every value it accepts is an exact integer, so a float cannot
// travel back.[^2] [^3]
//
// The script that closes the gap the lint leaves reads the core crate only,
// and that scope is correct.
#![allow(clippy::disallowed_types)]

pub mod hud;
pub mod metrics;
pub mod paint;
pub mod text;

pub use hud::Readout;
pub use metrics::{Lap, Metrics};
pub use paint::{Camera, Canvas};

use cachette_core::{BridgeError, World};

/// Draws one frame: the world, and then the panel that says what it holds.
///
/// This is the whole picture. The binary calls it, and so does every test
/// that checks the panel, because a panel that renders proves nothing until
/// something reaches it.[^1]
///
/// The order matters. The world pass clears the canvas and counts what it
/// paints, so the panel must read the canvas after that pass and draw over
/// it.
///
/// # Errors
///
/// Returns an error when the engine's spatial structure no longer describes
/// its soldiers. The viewer refuses rather than drawing a world without its
/// units and calling that a success.
///
/// # References
///
/// [^1]: Testing Rules, drive the real caller. `.claude/rules/testing.md`
pub fn draw_frame(
    world: &World,
    camera: Camera,
    metrics: &Metrics,
    canvas: &mut Canvas,
) -> Result<Readout, BridgeError> {
    paint::draw(world, camera, canvas)?;
    let readout = Readout::of(world, camera, canvas, metrics);
    hud::draw(&readout, canvas);
    Ok(readout)
}
