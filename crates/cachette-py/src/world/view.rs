//! What a front end needs to draw a frame, and nothing the engine reads.
//!
//! This module holds the panel and overlay names, the paint of one overlay,
//! the frame the engine draws into a caller's array, and the direction helpers
//! a front end uses to turn a key press into a step.
//!
//! The world holds none of this state. The engine keeps no camera, no timing
//! and no motion table, and the binding is the caller that keeps them.[^1]
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`

use super::{Presenter, PyWorld};
use crate::camera::PyCamera;
use crate::errors::{FrameError, ViewError};
use cachette_core::hex::NEIGHBOURS;
use cachette_core::TileIdx;
use cachette_core::{Axial, FactionId};
use cachette_view::panel::Set as PanelSet;
use cachette_view::{fill_frame_paced, Lap, Motion, Overlay, Pace, Surface};
use numpy::{PyReadwriteArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Returns the name of every panel the viewer can draw.
    ///
    /// A caller passes one of these names to `draw` as its `panels` argument.
    /// The list comes from the viewer's own registration. A panel that joins
    /// the deck appears here with no edit to this file.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[staticmethod]
    fn panel_names() -> Vec<&'static str> {
        cachette_view::panel::registered()
            .iter()
            .map(|panel| panel.name())
            .collect()
    }

    /// Gives back the name of every overlay the map can show, in its order.
    ///
    /// An overlay is one quantity of the world, drawn over every tile of the
    /// map as a strength of one colour. The map carries one at a time, and the
    /// caller names it on the frame command.
    ///
    /// **The engine holds the list and this derives from it**, so a caller
    /// that binds a key to each name binds one key to each overlay for as long
    /// as the list grows. A second list written here would be one fact in two
    /// places, with nothing to fail when the copies drift.[^1]
    ///
    /// **A presenter chooses among these names. It does not draw.** One
    /// renderer feeds every presenter, so a caller cannot name an overlay the
    /// renderer does not hold, and it cannot supply a rule of its own for
    /// painting one. A name taken from this list is therefore a choice among
    /// what the one renderer published, and not free text.[^2]
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^2]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    #[staticmethod]
    fn overlay_names() -> Vec<&'static str> {
        cachette_view::overlay::names()
    }

    /// Copies the paint of one overlay over every tile into two NumPy arrays.
    ///
    /// The name is one of the names that `overlay_names` gives back.
    ///
    /// Returns a `dict` with the keys `colour` and `strength`. The colour
    /// column holds `numpy.uint32`, one entry for each tile. The strength
    /// column holds `numpy.uint8`, one entry for each tile. Both stand in
    /// row-major order. Entry `r * width + q` is the tile at the address
    /// `(q, r)`. The order is the order that `tile_holders` uses, so these
    /// columns and the other tile columns index alike.
    ///
    /// **A colour is `0x00RRGGBB`, and a strength is a weight of 255.** The
    /// drawing pass mixes that colour into the ground of the tile at that
    /// weight, so a caller reproduces the pixel the map paints. A strength of
    /// zero paints nothing, and the colour beside it means nothing.
    ///
    /// **These two are the engine's own choices, and not the raw quantity.**
    /// An overlay owns its palette, and it owns the ramp from a value to a
    /// strength. The span of that ramp moves with the frame. A caller that
    /// took the quantity would hold a second copy of both, and nothing would
    /// fail when the copies drifted.[^1] A caller that wants the quantity
    /// reads the column that carries it.
    ///
    /// **The ramp is the ramp of this frame.** An overlay whose span runs
    /// between the lowest and the highest value of the world gives another
    /// strength after a step that moves either end.
    ///
    /// The answer is a pure function of the world and the name. This reads
    /// the world and writes nothing to it.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when no overlay carries the name. The message names
    /// the overlays that exist, in the way that `draw` does.
    ///
    /// # References
    ///
    /// [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^2]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    fn overlay_paint<'py>(&self, python: Python<'py>, name: &str) -> PyResult<Bound<'py, PyDict>> {
        let layer = cachette_view::overlay::named(name).ok_or_else(|| {
            ViewError::new_err(format!(
                "no overlay is called {name:?}; the overlays are {}",
                cachette_view::overlay::names().join(", ")
            ))
        })?;
        let (colours, strengths): (Vec<u32>, Vec<u8>) = python.detach(|| {
            let world = self.lock();
            let grid = world.grid();
            let span = layer.span(&world);
            (0..grid.tile_count())
                .map(|index| {
                    let Some(address) = grid.address_of(TileIdx(index)) else {
                        return (0, 0);
                    };
                    let value = cachette_view::overlay::value_of(layer, &world, address, None);
                    (layer.colour(value), layer.strength(value, span))
                })
                .unzip()
        });
        let paint = PyDict::new(python);
        paint.set_item("colour", colours.to_pyarray(python))?;
        paint.set_item("strength", strengths.to_pyarray(python))?;
        Ok(paint)
    }

    /// Fills the caller's pixels with one frame of this world.
    ///
    /// **The caller owns the memory before this call and owns it
    /// afterwards.** The engine writes each pixel of one frame into it and
    /// returns. It allocates no frame, keeps no frame, and holds no reference
    /// to the memory after the call ends.[^1]
    ///
    /// **This is one command and it carries no entity.** It takes a world, a
    /// camera and somewhere to put the result. It names no tile and no unit.
    /// A caller that walked tiles to draw them would cross the boundary once
    /// for each tile. The crossing costs more than the drawing.[^2]
    ///
    /// The camera says what part of the world the picture shows. The width
    /// and the height are the size of the picture in pixels.
    ///
    /// The pixels are a one-dimensional NumPy array of `numpy.uint32`. It
    /// must hold `width * height` entries and must be one contiguous block.
    /// Each entry holds red, green and blue in its low three bytes. Build one
    /// with `numpy.zeros(width * height, dtype=numpy.uint32)`, and reshape it
    /// to `(height, width)` after the call to show it.
    ///
    /// Set `reference` to show the layer that names the colours. Set `panel`
    /// to draw the whole panel instead of the cards, which is what a caller
    /// that writes a picture to a file wants. Set `panels` to draw the named
    /// panels as a deck beside the cards. A name that no panel carries is
    /// refused, and the message names the panels that exist. When both are
    /// set, the named panels win. Set `pointer` to an address, and the
    /// inspector panel of the deck reads that tile. It applies only when
    /// `panels` names a deck.
    ///
    /// Set `overlay` to the name of one overlay, and the map then shows that
    /// quantity over every tile as a strength of one colour. Read
    /// `overlay_names` for the names. A name that no overlay carries is
    /// refused, and the message names the overlays that exist. **A name from
    /// that list is a choice among what the one renderer published, and not
    /// free text**, so a presenter chooses an overlay and never draws one
    /// itself.[^5]
    ///
    /// The colour key names the overlay, the value at which it draws no
    /// colour, the value at which it draws full colour, and what the drawing
    /// pass met in the window. An overlay that found nothing says so in words.
    ///
    /// Set `phase` to the share of the current tick that has elapsed on the
    /// wall clock, from zero up to one. A unit that moved to a neighbouring
    /// tile since the last frame then draws between the two tile centres at
    /// that share. A unit new to the frame, and a unit that jumped further
    /// than a neighbouring tile, draws at its tile. A phase of zero draws
    /// every unit at its tile, and that is the default.
    ///
    /// Set `speed_milli` to the ticks each frame runs, in thousandths. Zero
    /// means paused. The frame states the speed as a word beside the tick,
    /// and the viewer holds the words, so the caller sends no text.[^5]
    ///
    /// **The engine keeps the table of where each unit was, and the world
    /// does not.** The world holds one tick at a time, so a frame drawn
    /// between two ticks needs a record of the first. The binding keeps that
    /// record beside the timing it already keeps, bounded by the pixels of
    /// the frame.[^6]
    ///
    /// Returns a `dict` of what the drawing pass read.[^3] A caller reports
    /// the numbers the picture was made from, and starts no second pass to
    /// find them.
    ///
    /// Most entries are plain integers. Eight are not. `promoted_deeds` may
    /// be `None`. `newest_character` is a pair of integers, or `None`.
    /// `centre` and `extent_shown` are pairs of integers. `carried_by_kind`
    /// is a list of one count for each resource kind. The three trailing
    /// entries are floating point numbers. `speed_says` is the word the
    /// frame stated for the speed.
    ///
    /// **The three floating point entries measure this machine and not the
    /// simulation.** `step_mean_micros` and `draw_mean_micros` are mean
    /// durations in microseconds, and `ticks_each_second` is a rate. Nothing
    /// in the engine reads them, and no two runs need to agree on them.
    ///
    /// **`rationed_short_accum` is a Q16.16 value as its raw integer.**
    /// Divide by 65536. The key names the unit, because a caller that read it
    /// as a count of goods would report a quantity 65536 times too large.
    /// Every other integer entry is a whole count.
    ///
    /// `panel_height` is the picture height that holds the whole panel. A
    /// caller that writes the panel to a file draws once, reads this entry,
    /// then draws again at that height.
    ///
    /// # Errors
    ///
    /// Raises `FrameError` when the array does not match the width and the
    /// height, when a side is zero, when the array is not contiguous, or when
    /// the camera draws a tile smaller than one pixel. The last refusal names
    /// the bound. Below one pixel per tile, a second tile falls on a pixel
    /// the first already holds. The work is provably invisible, and a caller
    /// could otherwise sweep the whole world for a picture of a few pixels.[^4]
    ///
    /// Raises `TypeError` when the array does not hold `numpy.uint32`
    /// entries. The interpreter refuses the argument before the engine reads
    /// it, so that refusal is not a `CachetteError`.
    ///
    /// # References
    ///
    /// [^1]: ADR-0094, the caller owns the camera and the pixels, decision D2. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    /// [^2]: ADR-0094, the caller owns the camera and the pixels, decision D1. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    /// [^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    /// [^4]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    /// [^5]: ADR-0094, the caller owns the camera and the pixels, decision D5. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
    /// [^6]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    // The arguments are the interface a Python caller types by name, so
    // bundling them would hide the contract rather than simplify it.
    //
    // The phase is a float, and the lint that bans the float types protects
    // simulated state. This value is a viewer value: it reaches the drawing,
    // it moves a unit between two tile centres, and no value formed from it
    // returns to the engine. The record that bans the type allows it
    // there.[^7]
    //
    // [^7]: ADR-0067, the viewer reads the world and never writes to it, decision D3. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[allow(clippy::disallowed_types)]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (camera, width, height, pixels, reference = false, panel = false, panels = None, pointer = None, overlay = None, phase = 0.0, speed_milli = 1000))]
    fn draw<'py>(
        &self,
        python: Python<'py>,
        camera: &PyCamera,
        width: usize,
        height: usize,
        pixels: PyReadwriteArray1<'py, u32>,
        reference: bool,
        panel: bool,
        panels: Option<Vec<String>>,
        pointer: Option<(i32, i32)>,
        overlay: Option<String>,
        phase: f32,
        speed_milli: u32,
    ) -> PyResult<Bound<'py, PyDict>> {
        let mut pixels = pixels;
        let buffer = pixels.as_slice_mut().map_err(|_| {
            FrameError::new_err(
                "the frame needs a contiguous array of unsigned 32-bit values".to_string(),
            )
        })?;

        let surface = Surface::new(width, height, buffer)
            .map_err(|error| FrameError::new_err(error.to_string()))?;

        // The named panels win over the whole panel, because a caller that
        // named a deck asked for the deck. A name that no panel carries is
        // refused, and the message names the panels that exist: a frame with
        // nothing on it looks the same as a frame the caller mistyped.
        let mut deck = PanelSet::EMPTY;
        for name in panels.unwrap_or_default() {
            deck = deck.with(&name).ok_or_else(|| {
                let known: Vec<&str> = cachette_view::panel::registered()
                    .iter()
                    .map(|panel| panel.name())
                    .collect();
                FrameError::new_err(format!(
                    "no panel is called {name:?}; the panels are {}",
                    known.join(", ")
                ))
            })?;
        }

        // The overlay the map shows, by name. A name the renderer did not
        // publish is refused, and the message names the overlays that exist.
        // A frame with no wash on it looks the same as a frame the caller
        // mistyped, so the engine says which happened.
        let layer = match overlay {
            None => None,
            Some(name) => Some(cachette_view::overlay::named(&name).ok_or_else(|| {
                FrameError::new_err(format!(
                    "no overlay is called {name:?}; the overlays are {}",
                    cachette_view::overlay::names().join(", ")
                ))
            })?),
        };

        let overlay = if !deck.is_empty() {
            Overlay::Deck {
                reference,
                panels: deck,
                pointer: pointer.map(|(q, r)| Axial { q, r }),
            }
        } else if panel {
            Overlay::Panel
        } else {
            Overlay::Glass { reference }
        };

        let at = Lap::start();
        let readout = {
            let world = self.lock();
            let mut presenter = self.presenter();
            // The table is bounded by the pixels of the frame, so it follows
            // the window and never the population. A frame of a new size gets
            // a table of that size, and the units of the frame before it draw
            // at their tiles once.
            if presenter.motion.bound() != width.saturating_mul(height) {
                presenter.motion = Motion::for_frame(width, height);
            }
            let Presenter {
                metrics,
                outcomes,
                motion,
            } = &mut *presenter;
            fill_frame_paced(
                &world,
                camera.inner,
                metrics,
                outcomes,
                overlay,
                layer,
                Pace::new(phase, speed_milli),
                motion,
                surface,
            )
            .map_err(|error| FrameError::new_err(error.to_string()))?
        };
        self.presenter().metrics.draw(at.elapsed());

        let report = PyDict::new(python);
        report.set_item("tick", readout.tick())?;
        report.set_item("tiles_painted", readout.tiles_painted())?;
        report.set_item("soldiers_painted", readout.soldiers_painted())?;
        report.set_item("soldiers_live", readout.soldiers_live())?;
        report.set_item("sites_held", readout.sites_held())?;
        // A position is a seat at a site. Both numbers come from the bounded
        // walk over the sites, so neither follows the world.
        report.set_item("seats", readout.seats())?;
        report.set_item("seats_taken", readout.seats_taken())?;
        // The character tier, walked once. A caller may walk this tier and
        // no other, because it holds a bounded population.
        report.set_item("characters", readout.characters())?;
        // How many units the last step promoted, and the deeds that earned
        // the first of them. Both come from the log of that step, so a caller
        // sees the promotion on the frame it happened rather than watching a
        // number go up.
        report.set_item("promoted_now", readout.promoted_now())?;
        report.set_item("promoted_deeds", readout.promoted_deeds())?;
        report.set_item(
            "newest_character",
            readout
                .newest_character()
                .map(|(faction, birth)| (faction.0, birth)),
        )?;
        // The height a picture needs to hold the whole panel. A caller that
        // writes the panel to a file resizes to this rather than guessing a
        // constant, because the panel grows with the faction count, with the
        // number of foundings, and with every section a count switches on.
        report.set_item("panel_height", readout.height_for_whole_panel())?;
        report.set_item("units_short", readout.units_short())?;
        // What the drawn units are hauling and where they live. Both are
        // counts of the window, taken on the loop that painted them, so a
        // caller reports them without starting a pass of its own.
        report.set_item("units_carrying", readout.units_carrying())?;
        report.set_item("carried_by_kind", *readout.carried_by_kind())?;
        report.set_item("units_housed", readout.units_housed())?;
        // The store of a site rations when it cannot serve its cohorts.
        // This is a count of the world, because the engine holds the log of
        // the step that just ran.
        report.set_item("sites_rationed", readout.rationings())?;
        // The shortfall is in accumulator units, which are fixed point at a
        // scale of 65536. The key names the unit, because a caller that read
        // this as a count of goods would report a quantity sixty-five
        // thousand times the real one.
        report.set_item("rationed_short_accum", readout.rationed_short())?;
        report.set_item("tiles_at_capacity", readout.tiles_at_capacity())?;
        report.set_item("crowd_worst", readout.crowd_worst())?;
        let centre = readout.centre();
        report.set_item("centre", (centre.q, centre.r))?;
        let (columns, rows) = readout.extent_shown();
        report.set_item("extent_shown", (columns, rows))?;
        // The speed the caller stated, and the word the viewer renders from
        // it. The engine holds no clock, so both come back from the number
        // the caller sent.
        report.set_item("speed_milli", readout.speed_milli())?;
        report.set_item("speed_says", readout.speed_word())?;
        report.set_item("step_mean_micros", readout.step_mean())?;
        report.set_item("draw_mean_micros", readout.draw_mean())?;
        report.set_item("ticks_each_second", readout.rate())?;
        // The overlay the frame drew, and the scale it drew at. The entry is
        // `None` when the caller named no overlay, so a caller cannot read a
        // scale that no picture used.
        report.set_item(
            "overlay",
            readout.overlay().map(|reading| {
                (
                    reading.name,
                    reading.span.low,
                    reading.span.high,
                    !reading.found_nothing(),
                )
            }),
        )?;
        Ok(report)
    }

    /// Returns the direction home for one faction at one address.
    ///
    /// The faction is a faction number of this world. The address is the pair
    /// `q` and `r`, as integers.
    ///
    /// The result is a direction, as an integer, or `None`. A direction is an
    /// index into the list that `direction_offsets` returns. The result is
    /// `None` when the block of ground holds a settlement of that faction.
    /// It is also `None` when no settlement of that faction is in reach.
    ///
    /// **The field answers for a block of ground and not for a tile.** The
    /// engine derives one direction for each faction and each block. Two
    /// addresses in one block therefore give one answer.[^1] The engine
    /// derives the field again at every step.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world, and when
    /// the world holds no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    fn return_direction(&self, faction: u16, q: i32, r: i32) -> PyResult<Option<u8>> {
        let world = self.lock();
        world
            .return_direction(FactionId(faction), Axial::new(q, r))
            .ok_or_else(|| {
                ViewError::new_err(format!(
                    "({q}, {r}) and the faction {faction} name no entry of the return field"
                ))
            })
    }

    /// Returns the offset of each direction, as a list of `(q, r)` pairs.
    ///
    /// A tile of this world has six neighbours. A direction is an index into
    /// this list. Add the pair at that index to an address to get the
    /// neighbour in that direction. The order never changes.[^1]
    ///
    /// The list is the one that the engine itself uses. The file declares
    /// the order nowhere else, so no second statement can disagree with it.
    ///
    /// Read `return_direction` for a direction to take.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[staticmethod]
    fn direction_offsets() -> Vec<(i32, i32)> {
        NEIGHBOURS
            .iter()
            .map(|offset| (offset.q, offset.r))
            .collect()
    }
}
