//! The panel that says what is happening.
//!
//! A person who watches coloured cells move can see that the world runs. They
//! cannot see which tick it is, where they are looking, who the colours are,
//! or what the step costs. This module writes those numbers over the picture.
//!
//! # What the panel may say
//!
//! Every number here comes from one of three places: a value the engine
//! already exposes, a value the viewer computed for itself, or a count the
//! drawing pass produced while it painted. The panel starts no pass of its
//! own over the world, and it asks the engine for nothing that the engine
//! does not already hold.[^1] [^2]
//!
//! That rule decides the content. The panel says how many soldiers stand in
//! the window, because the drawing pass counted them as it painted them. It
//! does not say how many soldiers each faction has in the whole world,
//! because nothing knows that without reading every soldier. A label says
//! which of the two a number is, so a reader never mistakes one for the
//! other.
//!
//! # The reading is separate from the writing
//!
//! A readout is a set of numbers. Painting a readout is a function of the
//! readout alone, so the same readout gives the same pixels. The reading
//! happens once, against the world and the finished canvas.
//!
//! # References
//!
//! [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
//! [^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/draft/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

use cachette_core::{Axial, World};

use crate::metrics::Metrics;
use crate::paint::{faction_colour, Camera, Canvas, COLOURED_FACTIONS};
use crate::text;

/// The gap between the window edge and the panel, in pixels.
const MARGIN: i32 = 14;

/// The width of the panel, in pixels.
const PANEL_WIDTH: i32 = 268;

/// The gap between the panel edge and its text, in pixels.
const PAD: i32 = 13;

/// The distance between one line of text and the next, in pixels.
const LINE: i32 = 12;

/// The offset of the value column from the panel text edge, in pixels.
const VALUE_COLUMN: i32 = 96;

/// The height of the bar that shows the faction shares, in pixels.
const BAR_HEIGHT: i32 = 5;

/// The colour of the panel, mixed over the world.
const PANEL: u32 = 0x0009_0e12;

/// How much of the panel colour covers the world under it.
const PANEL_WEIGHT: u8 = 224;

/// The colour of the panel edge and of the rules between sections.
const EDGE: u32 = 0x0027_3a44;

/// The colour of a section heading.
const HEADING: u32 = 0x0074_a6ba;

/// The colour of a label.
const LABEL: u32 = 0x0069_7d87;

/// The colour of a value.
const VALUE: u32 = 0x00d6_e4ea;

/// The colour of the title.
const TITLE: u32 = 0x00e8_c84a;

/// What the panel says.
///
/// A readout holds numbers and nothing else. It is read once against the
/// world, the camera and the finished canvas, and then it is only drawn.
#[derive(Clone, Debug)]
pub struct Readout {
    tick: u64,
    world_width: u32,
    world_height: u32,
    factions: u16,
    soldiers_live: u32,
    centre: Axial,
    tile_pixels: f32,
    columns_shown: u32,
    rows_shown: u32,
    tiles_painted: u32,
    soldiers_painted: u32,
    blocks_read: u32,
    blocks_skipped: u32,
    by_faction: [u32; COLOURED_FACTIONS],
    step_mean: f64,
    step_worst: f64,
    draw_mean: f64,
    draw_worst: f64,
    rate: f64,
    busy: f64,
}

impl Readout {
    /// Reads what the panel will say.
    ///
    /// Call this after the drawing pass. The canvas carries the counts of
    /// that pass, and a readout taken before it would report the pass before
    /// last.
    ///
    /// The world is a shared reference, so the compiler refuses a write to
    /// it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/draft/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
    #[must_use]
    pub fn of(world: &World, camera: Camera, canvas: &Canvas, metrics: &Metrics) -> Self {
        let grid = world.grid();
        let (first_row, last_row) = camera.visible_rows(world, canvas);
        let middle_row = (first_row + last_row) / 2;
        let (first_column, last_column) = camera.visible_columns(middle_row, world, canvas);

        Self {
            // The tick is the engine's own counter. The viewer reads it and
            // keeps no copy of its own, because two counters for one number
            // is one fact in two places.
            tick: world.tick().0,
            world_width: grid.width(),
            world_height: grid.height(),
            factions: world.config().faction_count,
            soldiers_live: world.soldiers().len(),
            centre: camera.tile_at(canvas.width() as f32 / 2.0, canvas.height() as f32 / 2.0),
            tile_pixels: camera.tile_width,
            columns_shown: last_column.saturating_sub(first_column),
            rows_shown: last_row.saturating_sub(first_row),
            tiles_painted: canvas.tiles_painted(),
            soldiers_painted: canvas.soldiers_painted(),
            blocks_read: canvas.blocks_read(),
            blocks_skipped: canvas.blocks_skipped(),
            by_faction: *canvas.painted_by_faction(),
            step_mean: metrics.step_mean_micros(),
            step_worst: metrics.step_worst_micros(),
            draw_mean: metrics.draw_mean_micros(),
            draw_worst: metrics.draw_worst_micros(),
            rate: metrics.ticks_each_second(),
            busy: metrics.busy_percent(),
        }
    }

    /// Returns the tick the engine has reached.
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    /// Returns the tile under the middle of the window.
    #[must_use]
    pub const fn centre(&self) -> Axial {
        self.centre
    }

    /// Returns the soldiers the world holds.
    #[must_use]
    pub const fn soldiers_live(&self) -> u32 {
        self.soldiers_live
    }

    /// Returns the soldiers the window shows.
    #[must_use]
    pub const fn soldiers_painted(&self) -> u32 {
        self.soldiers_painted
    }

    /// Returns the soldiers the window shows, one count for each colour.
    ///
    /// This is a census of the window. It is not a census of the world, and
    /// the panel labels it so.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/draft/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
    #[must_use]
    pub const fn by_faction(&self) -> &[u32; COLOURED_FACTIONS] {
        &self.by_faction
    }

    /// Returns the tiles the window shows, across and down.
    #[must_use]
    pub const fn extent_shown(&self) -> (u32, u32) {
        (self.columns_shown, self.rows_shown)
    }

    /// Returns the number of legend rows the panel draws.
    ///
    /// A faction beyond the colour table shares a colour with an earlier one,
    /// so the legend stops at the table and the panel says that it did.
    fn legend_rows(&self) -> usize {
        (self.factions as usize).clamp(1, COLOURED_FACTIONS)
    }

    /// Returns the height of the panel in pixels.
    ///
    /// The height follows the content, so a world with two factions gets a
    /// shorter panel than a world with six.
    fn height(&self) -> i32 {
        let sections = 4;
        let rows = 3 + 5 + self.legend_rows() as i32 + 5;
        PAD * 2 + 26 + sections * (LINE + 8) + rows * LINE + BAR_HEIGHT + 8 + LINE * 2
    }
}

/// Draws the panel over a canvas that already holds the world.
///
/// The function reads the readout and nothing else, so one readout always
/// gives one picture.
pub fn draw(readout: &Readout, canvas: &mut Canvas) {
    let (left, top, width, height) = bounds(readout);
    let text_left = left + PAD;
    let text_right = left + width - PAD;

    canvas.shade(left, top, width, height, PANEL, PANEL_WEIGHT);
    outline(canvas, left, top, width, height);

    let mut pen = top + PAD;

    canvas.write(text_left, pen, "CACHETTE", 2, TITLE);
    pen += 18;
    canvas.write(text_left, pen, "watching the world run", 1, LABEL);
    pen += LINE;

    pen = rule(canvas, text_left, text_right, pen);
    pen = heading(canvas, text_left, pen, "WORLD");
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "tick",
        &grouped(readout.tick),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "extent",
        &format!("{} x {} tiles", readout.world_width, readout.world_height),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "soldiers",
        &grouped(u64::from(readout.soldiers_live)),
    );

    pen = rule(canvas, text_left, text_right, pen);
    pen = heading(canvas, text_left, pen, "VIEW");
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "centre tile",
        &format!("q {}  r {}", readout.centre.q, readout.centre.r),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "zoom",
        &format!("{:.0} px a tile", readout.tile_pixels),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "showing",
        &format!("{} x {} tiles", readout.columns_shown, readout.rows_shown),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "tiles drawn",
        &grouped(u64::from(readout.tiles_painted)),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "units drawn",
        &grouped(u64::from(readout.soldiers_painted)),
    );

    pen = rule(canvas, text_left, text_right, pen);
    pen = heading(canvas, text_left, pen, "FACTIONS IN THE WINDOW");
    pen = legend(canvas, text_left, text_right, pen, readout);

    pen = rule(canvas, text_left, text_right, pen);
    pen = heading(canvas, text_left, pen, "COST ON THIS MACHINE");
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "step",
        &format!("{:.0} / {:.0} us", readout.step_mean, readout.step_worst),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "draw",
        &format!("{:.0} / {:.0} us", readout.draw_mean, readout.draw_worst),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "rate",
        &format!("{:.1} a second", readout.rate),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "busy",
        &format!("{:.0} in 100", readout.busy),
    );
    pen = row(
        canvas,
        text_left,
        text_right,
        pen,
        "blocks",
        &format!(
            "{} read, {} skipped",
            readout.blocks_read, readout.blocks_skipped
        ),
    );

    pen = rule(canvas, text_left, text_right, pen);
    canvas.write(text_left, pen, "mean and worst, one run, one", 1, LABEL);
    pen += LINE;
    canvas.write(text_left, pen, "machine. not the target.", 1, LABEL);
}

/// Returns the rectangle the panel occupies, as a left, top, width and
/// height in pixels.
///
/// The height follows the content, so a caller cannot know it without asking.
/// The panel writes nothing outside this rectangle, and a test reads it to
/// check that.
#[must_use]
pub fn bounds(readout: &Readout) -> (i32, i32, i32, i32) {
    (MARGIN, MARGIN, PANEL_WIDTH, readout.height())
}

/// Draws the panel edge.
fn outline(canvas: &mut Canvas, x: i32, y: i32, width: i32, height: i32) {
    canvas.block(x, y, width, 1, EDGE);
    canvas.block(x, y + height - 1, width, 1, EDGE);
    canvas.block(x, y, 1, height, EDGE);
    canvas.block(x + width - 1, y, 1, height, EDGE);
}

/// Draws a rule between two sections, and returns the next line position.
fn rule(canvas: &mut Canvas, left: i32, right: i32, pen: i32) -> i32 {
    canvas.block(left, pen + 3, right - left, 1, EDGE);
    pen + 8
}

/// Draws a section heading, and returns the next line position.
fn heading(canvas: &mut Canvas, left: i32, pen: i32, name: &str) -> i32 {
    canvas.write(left, pen, name, 1, HEADING);
    pen + LINE
}

/// Draws a label and its value, and returns the next line position.
///
/// The value sits against the right edge. A value too wide for the column is
/// cut rather than written over the panel edge, so text never escapes the
/// panel whatever a caller passes.
fn row(canvas: &mut Canvas, left: i32, right: i32, pen: i32, label: &str, value: &str) -> i32 {
    canvas.write(left, pen, label, 1, LABEL);

    let column = left + VALUE_COLUMN;
    let cells = ((right - column) / text::GLYPH_WIDTH).max(0) as usize;
    let value: String = value.chars().take(cells).collect();

    let start = column.max(right - text::width_of(&value, 1));
    canvas.write(start, pen, &value, 1, VALUE);
    pen + LINE
}

/// Draws the faction legend and the bar that shows the shares.
///
/// Each row names a colour, not a faction identity beyond the colour table.
/// A world with more factions than colours reuses a colour, and the row after
/// the legend says so.
fn legend(canvas: &mut Canvas, left: i32, right: i32, pen: i32, readout: &Readout) -> i32 {
    let rows = readout.legend_rows();
    let mut pen = pen;

    for (slot, count) in readout.by_faction.iter().enumerate().take(rows) {
        let colour = faction_colour(cachette_core::FactionId(slot as u16));
        canvas.block(left, pen, text::GLYPH_HEIGHT, text::GLYPH_HEIGHT, colour);
        canvas.write(left + 14, pen, &format!("faction {slot}"), 1, LABEL);
        let value = grouped(u64::from(*count));
        let width = text::width_of(&value, 1);
        canvas.write(right - width, pen, &value, 1, VALUE);
        pen += LINE;
    }

    // The bar shows the shares of what the window holds. A window with no
    // soldiers draws the empty bar rather than nothing, so the panel keeps
    // its shape and the reader sees that the answer is zero.
    let total: u32 = readout.by_faction.iter().take(rows).sum();
    let span = right - left;
    canvas.block(left, pen + 2, span, BAR_HEIGHT, EDGE);
    if total > 0 {
        let mut filled = 0;
        for (slot, count) in readout.by_faction.iter().enumerate().take(rows) {
            let share = (i64::from(*count) * i64::from(span) / i64::from(total)) as i32;
            let colour = faction_colour(cachette_core::FactionId(slot as u16));
            canvas.block(left + filled, pen + 2, share, BAR_HEIGHT, colour);
            filled += share;
        }
    }
    pen + BAR_HEIGHT + 8
}

/// Writes a number with a space between each group of three digits.
///
/// A tick count and a soldier count both reach six digits. A run of six
/// digits is hard to read at eight pixels a glyph.
fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}
