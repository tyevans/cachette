//! The frame command: one call fills a caller's pixels from a world.
//!
//! A caller supplies the memory, a camera, and a world. This module writes
//! one frame into that memory and returns. It allocates no frame, keeps no
//! frame, and holds no reference to the memory after the call ends.[^1]
//!
//! **The caller may be a Rust program or the control plane.** Both stand on
//! this command, and neither draws anything itself. A presenter that read the
//! world and drew part of a frame would be a second renderer, and two
//! renderers of one world disagree about that world with nothing to catch
//! it.[^2]
//!
//! The command carries no tile, no unit, and no entity. A caller that walked
//! tiles would cross the boundary once for each tile, and the crossing costs
//! more than the drawing.[^3]
//!
//! # References
//!
//! [^1]: ADR-0094, the caller owns the camera and the pixels, decision D2. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
//! [^2]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
//! [^3]: ADR-0040, Python is a control plane and not a data plane. `docs/adrs/REGISTRY.md`

use core::fmt;

use cachette_core::founding::FoundingOutcome;
use cachette_core::{BridgeError, World};

use crate::glass::Overlay;
use crate::hud::Readout;
use crate::metrics::Metrics;
use crate::paint::{Camera, Canvas};

/// The smallest tile size a frame will draw, in pixels on a side.
///
/// **This is a property of the pixel lattice, not a budget.** Below one pixel
/// for each tile, more than one tile falls on the same pixel, so the tiles
/// beyond the first cannot change what the frame holds. The work below this
/// scale is provably invisible. The bound follows from the lattice and from
/// nothing else, so it does not move because a picture looked better.[^1]
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
pub const LATTICE_BOUND: f32 = 1.0;

/// Why a frame was refused.
///
/// The engine is now wrong in public: the size of a frame and the scale of a
/// camera are part of the interface, so a caller that supplies the wrong one
/// is refused rather than trusted. A refusal costs a caller one error. A
/// write past the end of a caller's array does not.[^1]
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
#[derive(Debug, Clone, PartialEq)]
pub enum FrameError {
    /// The buffer does not hold one pixel for each pixel of the frame.
    BufferSize {
        /// The number of pixels the caller supplied.
        supplied: usize,
        /// The number of pixels the width and the height need.
        needed: usize,
    },
    /// A side of the frame is zero, so the frame holds no pixel.
    EmptyFrame {
        /// The width the caller asked for.
        width: usize,
        /// The height the caller asked for.
        height: usize,
    },
    /// The camera draws a tile smaller than one pixel.
    ///
    /// The frame refuses rather than holding the scale to the bound quietly.
    /// A silent hold returns a picture that does not match the camera the
    /// caller asked for, and a caller cannot tell that from a picture that
    /// does.
    ScaleBelowLattice {
        /// The tile width the camera carries, in pixels.
        tile_width: f32,
        /// The tile height the camera carries, in pixels.
        tile_height: f32,
    },
    /// The engine's spatial structure no longer describes its units.
    Bridge(BridgeError),
}

impl fmt::Display for FrameError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferSize { supplied, needed } => write!(
                out,
                "the frame needs {needed} pixels and the buffer holds {supplied}"
            ),
            Self::EmptyFrame { width, height } => write!(
                out,
                "a frame of {width} by {height} holds no pixel, so there is nothing to fill"
            ),
            Self::ScaleBelowLattice {
                tile_width,
                tile_height,
            } => write!(
                out,
                "a tile of {tile_width} by {tile_height} pixels is below the bound of \
                 {LATTICE_BOUND} pixel for each tile, where a second tile falls on a pixel \
                 the first already holds; draw the summary level instead of asking for a \
                 frame no pixel can show"
            ),
            Self::Bridge(error) => write!(out, "{error}"),
        }
    }
}

impl std::error::Error for FrameError {}

impl From<BridgeError> for FrameError {
    fn from(error: BridgeError) -> Self {
        Self::Bridge(error)
    }
}

/// The memory a caller lends the engine for one frame.
///
/// **The caller owns this before the call and owns it afterwards.** The
/// engine writes each pixel of one frame into it and returns. It allocates no
/// frame, keeps no frame, and holds no reference to the memory after the call
/// ends, and the borrow checker holds that rather than a comment.[^1]
///
/// Building one checks the size, so a frame that has a surface has a surface
/// that fits. A caller that supplies memory of the wrong size is refused
/// rather than trusted: a refusal is cheap, and a write past the end of a
/// caller's array is not.[^1]
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D2. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
pub struct Surface<'a> {
    width: usize,
    height: usize,
    pixels: &'a mut [u32],
}

impl<'a> Surface<'a> {
    /// Wraps a caller's pixels as a surface of this size.
    ///
    /// # Errors
    ///
    /// Returns `EmptyFrame` when a side is zero. Returns `BufferSize` when
    /// the slice does not hold one pixel for each pixel of the frame, and
    /// names the size it needed.
    pub fn new(width: usize, height: usize, pixels: &'a mut [u32]) -> Result<Self, FrameError> {
        if width == 0 || height == 0 {
            return Err(FrameError::EmptyFrame { width, height });
        }
        let needed = width.checked_mul(height).ok_or(FrameError::BufferSize {
            supplied: pixels.len(),
            needed: usize::MAX,
        })?;
        if pixels.len() != needed {
            return Err(FrameError::BufferSize {
                supplied: pixels.len(),
                needed,
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Returns the width in pixels.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Returns the height in pixels.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }
}

/// Fills a caller's pixels with one frame of a world.
///
/// The caller owns the surface before the call and owns it afterwards. This
/// function writes each pixel of the frame into the surface and returns. It
/// keeps no reference to the memory.[^1]
///
/// **The world is borrowed shared and the buffer is borrowed mutably.** They
/// are different objects. Nothing here changes simulated state, and no pixel
/// enters a state hash, so the record that forbids the drawing from writing
/// to the world still holds.[^2]
///
/// The returned readout is what the drawing pass read. It is the same reading
/// the panel laid out, so no number a caller reports can disagree with the
/// same number on the glass.[^3]
///
/// # Errors
///
/// Returns `ScaleBelowLattice` when the camera draws a tile smaller than one
/// pixel. Returns `Bridge` when the engine's spatial structure no longer
/// describes its units. The surface checked its own size when it was built.
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D2. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
/// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
/// [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
pub fn fill_frame(
    world: &World,
    camera: Camera,
    metrics: &Metrics,
    outcomes: &[FoundingOutcome],
    overlay: Overlay,
    surface: Surface<'_>,
) -> Result<Readout, FrameError> {
    // The cost of a frame follows the pixels in the surface. The scale check
    // runs before any pass over the world, so a refused frame reads no tile.
    if camera.tile_width < LATTICE_BOUND || camera.tile_height < LATTICE_BOUND {
        return Err(FrameError::ScaleBelowLattice {
            tile_width: camera.tile_width,
            tile_height: camera.tile_height,
        });
    }

    let (width, height) = (surface.width, surface.height);
    let mut canvas = Canvas::borrowing(surface.pixels, width, height);
    crate::draw_frame(world, camera, metrics, outcomes, overlay, &mut canvas)
        .map_err(FrameError::Bridge)
}
