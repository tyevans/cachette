//! The `Camera` class, which says what a picture shows.
//!
//! The camera is a value the control plane owns. The engine keeps no camera,
//! so a caller that draws holds one and passes it in.[^1]
//!
//! The class sits in its own module because it is the one type in this crate
//! that carries floating point numbers. The record that bans the float types
//! allows them for rendering, and the allowance is scoped to this file rather
//! than to the crate.[^2]
//!
//! # References
//!
//! [^1]: ADR-0094, the caller owns the camera and the pixels, decision D3. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
//! [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

use crate::world::PyWorld;
use cachette_core::Axial;
use cachette_view::{Camera, FrameSize};
use pyo3::prelude::*;

/// A camera the control plane owns, which says what a picture shows.
///
/// Build a camera, steer it, then pass it to `World.draw`. The camera holds
/// the size of one tile in pixels and the pixel offset of the tile at the
/// origin. Nothing else.
///
/// **A camera is not attached to a world.** One camera draws any world, and
/// two cameras draw one world. The camera verbs that need the size of the
/// picture take the width and the height as arguments.
///
/// **The camera is a presentation value, not simulation state.** The engine
/// holds no camera. It borrows one for the length of a draw call and keeps
/// nothing of it. A frame is a pure function of a world and a camera. Two
/// calls with the same world and the same camera give the same picture. That
/// property makes a scripted flight, an agent that steers, and a
/// reproducible screenshot possible.[^1]
///
/// The state lives in Python. Python decides when to move and by how much.
/// The pan share and the zoom step live here, once. A copy on both sides of
/// the boundary would be one value in two places. Nothing fails when the
/// copies disagree.[^2]
///
/// A verb that works in pixels alone takes no size. A camera verb reads no
/// pixel, so a caller that has not drawn yet can still steer.
///
/// # Build a camera
///
/// ```text
/// Camera(tile_size=None)
/// ```
///
/// **The parameters of the constructor are here, and not under a separate
/// entry.** The binding library does not publish the prose of a constructor.
/// This class doc comment is the one place that holds it.[^3]
///
/// - `tile_size`, a `float` or `None`. The width and the height of one tile,
///   in pixels. The default is `None`, which gives 12.0 pixels. The
///   constructor holds the value inside the range 2.0 to 64.0 pixels. A size
///   outside that range gives the nearest size inside it.
///
/// The tile size is a choice of the caller and not a property of the world.
/// The new camera sits at the corner of the world, so it shows the tile at
/// the address `(0, 0)`. Call `look_at` to place it somewhere else.
///
/// **Call the `fitting` static method to get the camera to start from.** A
/// caller that draws with it sees the world rather than an empty picture.
///
/// # References
///
/// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D3. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
/// [^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
/// [^3]: Findings register, FND-325. `docs/FINDINGS.md`
// **The camera is a viewer value, and the record that bans the float types
// allows them for rendering.**[^1] The allowance is scoped to this type and
// its methods, and not to the crate, because everything else in this file
// carries simulation values, which are exact by construction.
//
// A float cannot travel back into the engine from here. The camera is passed
// by value into a drawing call, the drawing borrows the world shared, and
// every value the engine accepts is an exact integer.[^2]
//
// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
// [^2]: ADR-0094, the caller owns the camera and the pixels, decision D3. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
#[allow(clippy::disallowed_types)]
#[pyclass(name = "Camera", module = "cachette._core", skip_from_py_object)]
#[derive(Clone, Copy)]
pub struct PyCamera {
    pub(crate) inner: Camera,
}

#[allow(clippy::disallowed_types)]
#[pymethods]
impl PyCamera {
    /// Builds a camera.
    ///
    /// **The prose for this call lives in the doc comment of the class.** The
    /// binding library does not copy the doc comment of a constructor onto the
    /// Python object. Prose written here reaches no reader of the published
    /// reference.[^1]
    ///
    /// # References
    ///
    /// [^1]: Findings register, FND-325. `docs/FINDINGS.md`
    #[new]
    #[pyo3(signature = (tile_size = None))]
    fn new(tile_size: Option<f32>) -> Self {
        Self {
            inner: tile_size.map_or_else(Camera::opening, Camera::at_tile_size),
        }
    }

    /// Returns a camera that fits the whole world into a picture of this
    /// size.
    ///
    /// The world is the world to fit. The width and the height are the size
    /// of the picture in pixels.
    ///
    /// **The size of one tile never falls below two pixels.** When the world
    /// is too large to fit at that size, the picture shows a part of the
    /// world.
    ///
    /// **This is the camera to start from.** A caller that draws with it sees
    /// the world rather than an empty picture.
    #[staticmethod]
    fn fitting(world: &PyWorld, width: usize, height: usize) -> Self {
        Self {
            inner: Camera::fitting(&world.lock(), &FrameSize::new(width, height)),
        }
    }

    /// The width of one tile in pixels, as a `float`.
    #[getter]
    const fn tile_width(&self) -> f32 {
        self.inner.tile_width
    }

    /// Sets the width of one tile in pixels, as a `float`.
    ///
    /// **The setter does not hold the value to any bound.** The caller owns
    /// the camera, so the caller may build any camera it likes. The frame
    /// verb refuses the ones it cannot draw. It names the bound it refuses
    /// against. A setter that held the scale quietly would return a picture
    /// that did not match the camera. A caller could not tell that from a
    /// picture that did.[^1]
    ///
    /// The scroll and zoom verbs do hold the scale. They are what a person
    /// drives. A person should not be able to press a key into a refusal.
    ///
    /// # References
    ///
    /// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    #[setter]
    const fn set_tile_width(&mut self, pixels: f32) {
        self.inner.tile_width = pixels;
    }

    /// The height of one tile in pixels, as a `float`.
    #[getter]
    const fn tile_height(&self) -> f32 {
        self.inner.tile_height
    }

    /// Sets the height of one tile in pixels, as a `float`.
    ///
    /// The setter holds the value to no bound, in the same way the width
    /// setter does. `World.draw` refuses a camera it cannot draw.
    #[setter]
    const fn set_tile_height(&mut self, pixels: f32) {
        self.inner.tile_height = pixels;
    }

    /// The pixel offset across the picture of the tile at the origin, as a
    /// `float`.
    ///
    /// The tile at the address `(0, 0)` is drawn at this offset. A larger
    /// value moves the world to the right in the picture.
    #[getter]
    const fn origin_x(&self) -> f32 {
        self.inner.origin_x
    }

    /// Sets the pixel offset across the picture of the tile at the origin, as
    /// a `float`.
    #[setter]
    const fn set_origin_x(&mut self, pixels: f32) {
        self.inner.origin_x = pixels;
    }

    /// The pixel offset down the picture of the tile at the origin, as a
    /// `float`.
    ///
    /// A larger value moves the world down in the picture.
    #[getter]
    const fn origin_y(&self) -> f32 {
        self.inner.origin_y
    }

    /// Sets the pixel offset down the picture of the tile at the origin, as a
    /// `float`.
    #[setter]
    const fn set_origin_y(&mut self, pixels: f32) {
        self.inner.origin_y = pixels;
    }

    /// Moves the view by whole presses of a scroll key. Returns `None`.
    ///
    /// The across value and the down value are counts of presses, as
    /// `float` values. Pass minus one, zero or one for each direction. The
    /// width and the height are the size of the picture in pixels.
    ///
    /// **This is the call a person drives.** The step is a share of the
    /// picture. One press moves the view by the same part of it at every
    /// zoom.
    fn nudge(&mut self, across: f32, down: f32, width: usize, height: usize) {
        self.inner = self
            .inner
            .nudged(across, down, &FrameSize::new(width, height));
    }

    /// Moves the view by a count of pixels. Returns `None`.
    ///
    /// The across value and the down value are counts of pixels, as `float`
    /// values. The call needs no picture size, because a pixel is a pixel at
    /// every zoom.
    fn pan(&mut self, across: f32, down: f32) {
        self.inner = self.inner.panned(across, down);
    }

    /// Makes each tile larger by one press. Returns `None`.
    ///
    /// The width and the height are the size of the picture in pixels. The
    /// tile under the middle of the picture stays under the middle. The
    /// call holds the tile size inside the range the camera accepts.
    fn zoom_in(&mut self, width: usize, height: usize) {
        self.inner = self.inner.zoomed_in(&FrameSize::new(width, height));
    }

    /// Makes each tile smaller by one press. Returns `None`.
    ///
    /// The width and the height are the size of the picture in pixels. The
    /// tile under the middle of the picture stays under the middle. The
    /// call holds the tile size inside the range the camera accepts.
    fn zoom_out(&mut self, width: usize, height: usize) {
        self.inner = self.inner.zoomed_out(&FrameSize::new(width, height));
    }

    /// Puts a tile in the middle of the picture. Returns `None`.
    ///
    /// The address is the column `q` and the row `r`. The width and the
    /// height are the size of the picture in pixels. The call changes the
    /// offset alone, and it changes no tile size.
    fn look_at(&mut self, q: i32, r: i32, width: usize, height: usize) {
        self.inner = self
            .inner
            .looking_at(Axial::new(q, r), &FrameSize::new(width, height));
    }

    /// Holds the view so the world cannot leave the picture. Returns `None`.
    ///
    /// The world is the world whose bounds hold the view. The width and
    /// the height are the size of the picture in pixels. In each direction,
    /// at least half of the smaller of the world and the picture stays on
    /// the screen.
    ///
    /// A camera that ran off the edge would show a picture of nothing. A
    /// person could not tell that from an empty world.
    fn clamp(&mut self, world: &PyWorld, width: usize, height: usize) {
        self.inner = self
            .inner
            .clamped(&world.lock(), &FrameSize::new(width, height));
    }

    /// Returns the tile under a pixel, as a `tuple` of two integers.
    ///
    /// The pixel offsets are `float` values, across and then down. The result
    /// is the column `q` and the row `r`.
    ///
    /// **The result may lie outside the world.** The call converts a pixel
    /// into an address and reads nothing. Check the pair against `width`
    /// and `height` before you pass it on.
    ///
    /// **This is how a click reaches a tile without a loop.** The control
    /// plane sends one pixel and gets one address, rather than walking the
    /// tiles to find which one was hit.
    fn tile_at(&self, x: f32, y: f32) -> (i32, i32) {
        let address = self.inner.tile_at(x, y);
        (address.q, address.r)
    }

    /// Returns a `str` that names the camera with its tile size and its
    /// origin.
    fn __repr__(&self) -> String {
        format!(
            "Camera(tile_width={}, tile_height={}, origin_x={}, origin_y={})",
            self.inner.tile_width, self.inner.tile_height, self.inner.origin_x, self.inner.origin_y
        )
    }
}
