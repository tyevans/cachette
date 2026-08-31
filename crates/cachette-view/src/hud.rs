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
}

/// One line of the panel.
///
/// The panel is a list of these. The list is built once and it is the only
/// statement of what the panel holds. The height is summed from it and the
/// painting walks it, so the two cannot disagree.
///
/// An earlier version stated the height with its own arithmetic while the
/// painting produced the same geometry line by line. That is one fact in two
/// places, and nothing failed when the copies drifted.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
#[derive(Clone, Debug)]
enum Line {
    /// The name of the viewer, at twice the glyph size.
    Title(&'static str),
    /// A dim line that runs the width of the panel.
    Note(&'static str),
    /// A hairline between two sections.
    Rule,
    /// The name of a section.
    Heading(&'static str),
    /// A label on the left and a value against the right edge.
    Row(&'static str, String),
    /// A colour swatch, the faction it stands for, and its count.
    Legend(usize, u32),
    /// The bar that shows the shares of the visible units.
    Bar,
}

impl Line {
    /// Returns the height this line occupies, in pixels.
    const fn height(&self) -> i32 {
        match self {
            Self::Title(_) => 18,
            Self::Note(_) | Self::Heading(_) | Self::Row(_, _) | Self::Legend(_, _) => LINE,
            Self::Rule => 8,
            Self::Bar => BAR_HEIGHT + 8,
        }
    }
}

impl Readout {
    /// Builds the lines the panel holds.
    ///
    /// This is the whole content of the panel, in order. Nothing else states
    /// what the panel says.
    ///
    /// Each label names one thing, and no two labels name the same thing with
    /// different words. A reader must be able to tell a count of the world
    /// from a count of the window by the label alone, never by the section it
    /// sits under.
    fn lines(&self) -> Vec<Line> {
        let mut lines = vec![
            Line::Title("CACHETTE"),
            Line::Note("watching the world run"),
            Line::Rule,
            Line::Heading("WORLD"),
            Line::Row("tick", grouped(self.tick)),
            Line::Row(
                "extent",
                format!("{} x {} tiles", self.world_width, self.world_height),
            ),
            Line::Row("units alive", grouped(u64::from(self.soldiers_live))),
            Line::Rule,
            Line::Heading("VIEW"),
            Line::Row(
                "centre tile",
                format!("q {}  r {}", self.centre.q, self.centre.r),
            ),
            Line::Row("zoom", format!("{:.0} px a tile", self.tile_pixels)),
            Line::Row(
                "showing",
                format!("{} x {} tiles", self.columns_shown, self.rows_shown),
            ),
            Line::Row("tiles drawn", grouped(u64::from(self.tiles_painted))),
            Line::Row("units drawn", grouped(u64::from(self.soldiers_painted))),
            Line::Rule,
            Line::Heading("FACTIONS IN THE WINDOW"),
        ];

        for (slot, count) in self.by_faction.iter().enumerate().take(self.legend_rows()) {
            lines.push(Line::Legend(slot, *count));
        }
        lines.push(Line::Bar);

        lines.extend([
            Line::Rule,
            Line::Heading("COST ON THIS MACHINE"),
            Line::Row(
                "step",
                format!("{:.0} / {:.0} us", self.step_mean, self.step_worst),
            ),
            Line::Row(
                "draw",
                format!("{:.0} / {:.0} us", self.draw_mean, self.draw_worst),
            ),
            Line::Row("rate", format!("{:.1} a second", self.rate)),
            Line::Row("busy", format!("{:.0} in 100", self.busy)),
            // Two rows, not one. A single row that named both counts did not
            // fit the value column at every zoom, and the clip that kept it
            // inside the panel cut the last word in half.
            Line::Row("blocks read", grouped(u64::from(self.blocks_read))),
            Line::Row("blocks skipped", grouped(u64::from(self.blocks_skipped))),
            Line::Rule,
            Line::Note("mean and worst, one run, one"),
            Line::Note("machine. not the target."),
        ]);

        lines
    }

    /// Returns the height of the panel in pixels.
    ///
    /// The height is the sum of the lines. It follows the content, so a world
    /// with two factions gets a shorter panel than a world with six.
    fn height(&self) -> i32 {
        PAD * 2 + self.lines().iter().map(Line::height).sum::<i32>()
    }
}

/// Returns the width in pixels that the value column gives a value.
const fn value_span() -> i32 {
    PANEL_WIDTH - PAD * 2 - VALUE_COLUMN
}

/// Returns every value the panel would have to cut to fit its column.
///
/// The panel cuts a value that does not fit, so that text can never be
/// written over the panel edge. A cut value is still a defect: it states
/// something other than the number it was given, and it does so silently.
///
/// This function is how a test sees the cut. A test that only checked the
/// panel edge would pass because of the cut rather than in spite of it.
#[must_use]
pub fn values_that_do_not_fit(readout: &Readout) -> Vec<String> {
    readout
        .lines()
        .iter()
        .filter_map(|line| match line {
            Line::Row(_, value) if !value_fits(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

/// Says whether a value fits the column the panel gives it.
///
/// A test calls this on a string of its own, to prove that the check above
/// can answer no. A check with no proven failure mode is decoration.[^1]
///
/// # References
///
/// [^1]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`
#[must_use]
pub fn value_fits(value: &str) -> bool {
    text::width_of(value, 1) <= value_span()
}

/// Draws the panel over a canvas that already holds the world.
///
/// The function reads the readout and nothing else, so one readout always
/// gives one picture.
///
/// It walks the same list of lines that the height was summed from, so the
/// panel cannot paint past the rectangle it states.
pub fn draw(readout: &Readout, canvas: &mut Canvas) {
    let (left, top, width, height) = bounds(readout);
    let text_left = left + PAD;
    let text_right = left + width - PAD;

    canvas.shade(left, top, width, height, PANEL, PANEL_WEIGHT);
    outline(canvas, left, top, width, height);

    let lines = readout.lines();
    let mut pen = top + PAD;
    for line in &lines {
        paint_line(canvas, text_left, text_right, pen, line, readout);
        pen += line.height();
    }
}

/// Paints one line at the given position.
fn paint_line(
    canvas: &mut Canvas,
    left: i32,
    right: i32,
    pen: i32,
    line: &Line,
    readout: &Readout,
) {
    match line {
        Line::Title(name) => {
            canvas.write(left, pen, name, 2, TITLE);
        }
        Line::Note(note) => {
            canvas.write(left, pen, note, 1, LABEL);
        }
        Line::Rule => canvas.block(left, pen + 3, right - left, 1, EDGE),
        Line::Heading(name) => {
            canvas.write(left, pen, name, 1, HEADING);
        }
        Line::Row(label, value) => row(canvas, left, right, pen, label, value),
        Line::Legend(slot, count) => legend_row(canvas, left, right, pen, *slot, *count),
        Line::Bar => bar(canvas, left, right, pen, readout),
    }
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

/// Draws a label and its value.
///
/// The value sits against the right edge. A value too wide for the column is
/// cut rather than written over the panel edge, so text never escapes the
/// panel whatever a caller passes.
///
/// The cut is a guard, not a layout. A value that reaches it states something
/// other than the number it was given, and a test reads the same lines to
/// find one.
fn row(canvas: &mut Canvas, left: i32, right: i32, pen: i32, label: &str, value: &str) {
    canvas.write(left, pen, label, 1, LABEL);

    let column = left + VALUE_COLUMN;
    let cells = ((right - column) / text::GLYPH_WIDTH).max(0) as usize;
    let value: String = value.chars().take(cells).collect();

    let start = column.max(right - text::width_of(&value, 1));
    canvas.write(start, pen, &value, 1, VALUE);
}

/// Draws one legend row: a colour swatch, the faction, and its count.
///
/// The row names a colour, not a faction identity beyond the colour table. A
/// world with more factions than colours reuses a colour.
fn legend_row(canvas: &mut Canvas, left: i32, right: i32, pen: i32, slot: usize, count: u32) {
    let colour = faction_colour(cachette_core::FactionId(slot as u16));
    canvas.block(left, pen, text::GLYPH_HEIGHT, text::GLYPH_HEIGHT, colour);
    canvas.write(left + 14, pen, &format!("faction {slot}"), 1, LABEL);
    let value = grouped(u64::from(count));
    canvas.write(right - text::width_of(&value, 1), pen, &value, 1, VALUE);
}

/// Draws the bar that shows the shares of the units in the window.
///
/// A window with no unit draws the empty bar rather than nothing, so the
/// panel keeps its shape and the reader sees that the answer is zero.
fn bar(canvas: &mut Canvas, left: i32, right: i32, pen: i32, readout: &Readout) {
    let rows = readout.legend_rows();
    let total: u32 = readout.by_faction.iter().take(rows).sum();
    let span = right - left;
    canvas.block(left, pen + 2, span, BAR_HEIGHT, EDGE);
    if total == 0 {
        return;
    }
    let mut filled = 0;
    for (slot, count) in readout.by_faction.iter().enumerate().take(rows) {
        let share = (i64::from(*count) * i64::from(span) / i64::from(total)) as i32;
        let colour = faction_colour(cachette_core::FactionId(slot as u16));
        canvas.block(left + filled, pen + 2, share, BAR_HEIGHT, colour);
        filled += share;
    }
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
